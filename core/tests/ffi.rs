use std::ffi::CString;
use spoofer_core::*;

fn last_error() -> String {
    let mut buf = vec![0i8; 256];
    unsafe { gp_last_error(buf.as_mut_ptr() as *mut _, buf.len()) };
    let bytes: Vec<u8> = buf.iter().take_while(|b| **b != 0).map(|b| *b as u8).collect();
    String::from_utf8(bytes).unwrap()
}

#[test]
fn error_paths() {
    let ip = CString::new("127.0.0.1").unwrap();

    // no session yet
    assert_eq!(gp_set_location(48.85, 2.29), GP_ERR_NOT_CONNECTED);
    assert_eq!(gp_set_location(95.0, 2.29), GP_ERR_ARGS);
    assert_eq!(gp_state(), GP_STATE_IDLE);

    // missing pairing file
    let missing = CString::new("/nonexistent/pairing.plist").unwrap();
    assert_eq!(unsafe { gp_connect(missing.as_ptr(), ip.as_ptr(), 49152) }, GP_ERR_PAIRING_FILE);
    assert!(last_error().contains("read pairing file"), "{}", last_error());
    assert_eq!(unsafe { gp_pairing_file_valid(missing.as_ptr()) }, 0);

    // garbage pairing file
    let dir = std::env::temp_dir();
    let bad = dir.join("gp_bad.plist");
    std::fs::write(&bad, b"not a plist").unwrap();
    let bad_c = CString::new(bad.to_str().unwrap()).unwrap();
    assert_eq!(unsafe { gp_connect(bad_c.as_ptr(), ip.as_ptr(), 49152) }, GP_ERR_PAIRING_FILE);
    assert_eq!(unsafe { gp_pairing_file_valid(bad_c.as_ptr()) }, 0);

    // valid pairing file, nothing listening (VPN off)
    let good = dir.join("gp_good.plist");
    let rpf = idevice::remote_pairing::RpPairingFile::generate("test");
    std::fs::write(&good, rpf.to_bytes()).unwrap();
    let good_c = CString::new(good.to_str().unwrap()).unwrap();
    assert_eq!(unsafe { gp_pairing_file_valid(good_c.as_ptr()) }, 1);
    assert_eq!(unsafe { gp_connect(good_c.as_ptr(), ip.as_ptr(), 1) }, GP_ERR_UNREACHABLE);
    assert!(last_error().contains("connect 127.0.0.1:1"), "{}", last_error());

    // a listener that is not an iPhone: pairing must fail cleanly, not hang
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((s, _)) = listener.accept() {
            drop(s);
        }
    });
    assert_eq!(unsafe { gp_connect(good_c.as_ptr(), ip.as_ptr(), port) }, GP_ERR_PAIRING_REJECTED);

    // bad ip
    let bad_ip = CString::new("not-an-ip").unwrap();
    assert_eq!(unsafe { gp_connect(good_c.as_ptr(), bad_ip.as_ptr(), 1) }, GP_ERR_ARGS);
    gp_disconnect();
    assert_eq!(gp_clear_location(), GP_ERR_NOT_CONNECTED);
}
