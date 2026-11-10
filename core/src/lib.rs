//! On-device core of the iPhone app.
//!
//! The app talks to its own iPhone exactly like a computer would, but through
//! a loopback VPN (LocalDevVPN) that makes the device reachable at 10.7.0.1:
//!
//!   pairing file -> RemotePairing (verify) -> TLS-PSK tunnel -> RSD
//!   -> DVT instruments -> location simulation
//!
//! Everything is exposed to Swift through a handful of blocking C functions.
//! One background task owns the whole connection; Swift sends it commands.

use std::ffi::{CStr, c_char};
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;
use std::sync::mpsc as std_mpsc;
use std::time::Duration;

use idevice::{IdeviceError, RsdService};
use idevice::dvt::location_simulation::LocationSimulationClient;
use idevice::dvt::remote_server::RemoteServerClient;
use idevice::remote_pairing::{
    RemotePairingClient, RpPairingFile, RpPairingSocket, connect_tls_psk_tunnel_native,
};
use idevice::rsd::RsdHandshake;
use tokio::sync::mpsc;

// ---- result codes shared with Swift (see Bridge.h) -------------------------
pub const GP_OK: i32 = 0;
pub const GP_ERR_ARGS: i32 = 1;
pub const GP_ERR_PAIRING_FILE: i32 = 2; // file missing or unreadable
pub const GP_ERR_UNREACHABLE: i32 = 3; // VPN off / wrong port
pub const GP_ERR_PAIRING_REJECTED: i32 = 4; // device refused the pairing file
pub const GP_ERR_TUNNEL: i32 = 5;
pub const GP_ERR_DVT: i32 = 6; // developer image not mounted / dev mode off
pub const GP_ERR_NOT_CONNECTED: i32 = 7;
pub const GP_ERR_COMMAND: i32 = 8; // set/clear failed, connection dropped
pub const GP_ERR_TIMEOUT: i32 = 9;

pub const GP_STATE_IDLE: i32 = 0;
pub const GP_STATE_CONNECTED: i32 = 1;
pub const GP_STATE_LOST: i32 = 2;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(12);
const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);
const KEEPALIVE: Duration = Duration::from_secs(4);

enum Command {
    Set(f64, f64, std_mpsc::Sender<Result<(), String>>),
    Clear(std_mpsc::Sender<Result<(), String>>),
    Stop,
}

static SENDER: Mutex<Option<mpsc::Sender<Command>>> = Mutex::new(None);
static LAST_ERROR: Mutex<String> = Mutex::new(String::new());
static STATE: AtomicI32 = AtomicI32::new(GP_STATE_IDLE);

fn set_error(msg: impl Into<String>) {
    if let Ok(mut e) = LAST_ERROR.lock() {
        *e = msg.into();
    }
}

struct Failure {
    code: i32,
    msg: String,
}

fn fail(code: i32, what: &str, e: impl std::fmt::Debug) -> Failure {
    Failure {
        code,
        msg: format!("{what}: {e:?}"),
    }
}

/// Opens the tunnel and runs until told to stop or the connection drops.
async fn session(
    pairing_path: String,
    addr: SocketAddr,
    ready: std_mpsc::Sender<Result<(), Failure>>,
    mut rx: mpsc::Receiver<Command>,
) {
    // --- 1. pairing file ---------------------------------------------------
    let original = match std::fs::read(&pairing_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            let _ = ready.send(Err(fail(GP_ERR_PAIRING_FILE, "read pairing file", e)));
            return;
        }
    };
    let mut rpf = match RpPairingFile::from_bytes(&original) {
        Ok(f) => f,
        Err(e) => {
            let _ = ready.send(Err(fail(GP_ERR_PAIRING_FILE, "parse pairing file", e)));
            return;
        }
    };

    // --- 2. reach the device through the loopback VPN ----------------------
    let stream = match tokio::time::timeout(CONNECT_TIMEOUT, tokio::net::TcpStream::connect(addr)).await
    {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            let _ = ready.send(Err(fail(GP_ERR_UNREACHABLE, &format!("connect {addr}"), e)));
            return;
        }
        Err(_) => {
            let _ = ready.send(Err(fail(GP_ERR_UNREACHABLE, &format!("connect {addr}"), "timeout")));
            return;
        }
    };

    // --- 3. pair-verify with the stored identity ---------------------------
    let host = "LocationSpoofer";
    let mut rpc = RemotePairingClient::new(RpPairingSocket::new(stream), host);
    if let Err(e) = rpc.connect(&mut rpf, async || "000000".to_string()).await {
        let _ = ready.send(Err(fail(GP_ERR_PAIRING_REJECTED, "remote pairing", e)));
        return;
    }
    // The library may refresh the file in place; keep the stored copy in sync.
    let updated = rpf.to_bytes();
    if updated != original {
        let _ = std::fs::write(&pairing_path, &updated);
    }

    // --- 4. TLS-PSK tunnel + RSD -------------------------------------------
    let tunnel = async {
        let port = rpc.create_tcp_listener().await?;
        let mut tunnel_addr = addr;
        tunnel_addr.set_port(port);
        let stream = tokio::time::timeout(CONNECT_TIMEOUT, tokio::net::TcpStream::connect(tunnel_addr))
            .await
            .map_err(|_| IdeviceError::InternalError("tunnel connect timeout".into()))?
            .map_err(|e| IdeviceError::InternalError(format!("tunnel connect: {e}")))?;
        let tunnel = connect_tls_psk_tunnel_native(stream, rpc.encryption_key()).await?;
        let client_ip: IpAddr = tunnel
            .info
            .client_address
            .parse()
            .map_err(|e| IdeviceError::InternalError(format!("client ip: {e}")))?;
        let server_ip: IpAddr = tunnel
            .info
            .server_address
            .parse()
            .map_err(|e| IdeviceError::InternalError(format!("server ip: {e}")))?;
        let mtu = tunnel.info.mtu as usize;
        let rsd_port = tunnel.info.server_rsd_port;
        let mut adapter =
            idevice::tcp::adapter::Adapter::new(Box::new(tunnel.into_inner()), client_ip, server_ip);
        adapter.set_mss(mtu.min(8192) - 60);
        let mut adapter = adapter.to_async_handle();
        let rsd_stream = adapter
            .connect(rsd_port)
            .await
            .map_err(|e| IdeviceError::InternalError(format!("rsd connect: {e}")))?;
        let handshake = RsdHandshake::new(rsd_stream).await?;
        Ok::<_, IdeviceError>((adapter, handshake))
    }
    .await;
    let (mut adapter, mut handshake) = match tunnel {
        Ok(t) => t,
        Err(e) => {
            let _ = ready.send(Err(fail(GP_ERR_TUNNEL, "tunnel", e)));
            return;
        }
    };

    // --- 5. DVT instruments + location simulation channel ------------------
    let mut server = match RemoteServerClient::connect_rsd(&mut adapter, &mut handshake).await {
        Ok(s) => s,
        Err(e) => {
            let _ = ready.send(Err(fail(GP_ERR_DVT, "instruments service", e)));
            return;
        }
    };
    if let Err(e) = server.read_message(0).await {
        let _ = ready.send(Err(fail(GP_ERR_DVT, "instruments hello", e)));
        return;
    }
    let mut location = match LocationSimulationClient::new(&mut server).await {
        Ok(l) => l,
        Err(e) => {
            let _ = ready.send(Err(fail(GP_ERR_DVT, "location channel", e)));
            return;
        }
    };

    STATE.store(GP_STATE_CONNECTED, Ordering::SeqCst);
    let _ = ready.send(Ok(()));

    // --- 6. command loop ----------------------------------------------------
    // The simulated location only holds while this session is alive, and the
    // last point is re-sent regularly so the link never goes idle.
    let mut current: Option<(f64, f64)> = None;
    loop {
        match tokio::time::timeout(KEEPALIVE, rx.recv()).await {
            Ok(Some(Command::Set(lat, lon, reply))) => {
                let res = location.set(lat, lon).await;
                let failed = res.is_err();
                let _ = reply.send(res.map_err(|e| format!("{e:?}")));
                if failed {
                    break;
                }
                current = Some((lat, lon));
            }
            Ok(Some(Command::Clear(reply))) => {
                let res = location.clear().await;
                let failed = res.is_err();
                let _ = reply.send(res.map_err(|e| format!("{e:?}")));
                if failed {
                    break;
                }
                current = None;
            }
            Ok(Some(Command::Stop)) | Ok(None) => {
                let _ = location.clear().await;
                STATE.store(GP_STATE_IDLE, Ordering::SeqCst);
                return;
            }
            Err(_) => {
                if let Some((lat, lon)) = current {
                    if let Err(e) = location.set(lat, lon).await {
                        set_error(format!("keepalive: {e:?}"));
                        break;
                    }
                }
            }
        }
    }
    STATE.store(GP_STATE_LOST, Ordering::SeqCst);
}

unsafe fn cstr(p: *const c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(p) }.to_str().ok().map(str::to_owned)
}

fn drop_session() {
    if let Ok(mut guard) = SENDER.lock() {
        if let Some(tx) = guard.take() {
            let _ = tx.try_send(Command::Stop);
        }
    }
}

/// Connects to the device. `ip` is normally "10.7.0.1" (LocalDevVPN) and
/// `port` the RemotePairing port (49152 unless discovered otherwise).
///
/// # Safety
/// `pairing_path` and `ip` must be valid NUL-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gp_connect(pairing_path: *const c_char, ip: *const c_char, port: u16) -> i32 {
    let (Some(path), Some(ip)) = (unsafe { cstr(pairing_path) }, unsafe { cstr(ip) }) else {
        set_error("invalid arguments");
        return GP_ERR_ARGS;
    };
    let Ok(ip) = ip.parse::<IpAddr>() else {
        set_error(format!("invalid ip: {ip}"));
        return GP_ERR_ARGS;
    };
    drop_session();
    STATE.store(GP_STATE_IDLE, Ordering::SeqCst);

    let (tx, rx) = mpsc::channel(16);
    let (ready_tx, ready_rx) = std_mpsc::channel();
    let addr = SocketAddr::new(ip, port);
    // The session owns non-Send borrows, so it gets its own thread and a
    // single-threaded runtime instead of being spawned on a shared one.
    let spawned = std::thread::Builder::new()
        .name("spoofer-session".into())
        .spawn(move || {
            match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(rt) => rt.block_on(session(path, addr, ready_tx, rx)),
                Err(e) => {
                    let _ = ready_tx.send(Err(fail(GP_ERR_TUNNEL, "runtime", e)));
                }
            }
        });
    if let Err(e) = spawned {
        set_error(format!("thread: {e}"));
        return GP_ERR_TUNNEL;
    }

    match ready_rx.recv_timeout(Duration::from_secs(45)) {
        Ok(Ok(())) => {
            if let Ok(mut guard) = SENDER.lock() {
                *guard = Some(tx);
            }
            set_error("");
            GP_OK
        }
        Ok(Err(f)) => {
            set_error(f.msg);
            f.code
        }
        Err(std_mpsc::RecvTimeoutError::Timeout) => {
            set_error("connection timed out");
            GP_ERR_TIMEOUT
        }
        Err(std_mpsc::RecvTimeoutError::Disconnected) => {
            set_error("session ended unexpectedly");
            GP_ERR_TUNNEL
        }
    }
}

fn send_command(make: impl FnOnce(std_mpsc::Sender<Result<(), String>>) -> Command) -> i32 {
    let tx = match SENDER.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => None,
    };
    let Some(tx) = tx else {
        set_error("not connected");
        return GP_ERR_NOT_CONNECTED;
    };
    let (reply_tx, reply_rx) = std_mpsc::channel();
    if tx.try_send(make(reply_tx)).is_err() {
        set_error("connection closed");
        return GP_ERR_COMMAND;
    }
    match reply_rx.recv_timeout(COMMAND_TIMEOUT) {
        Ok(Ok(())) => GP_OK,
        Ok(Err(msg)) => {
            set_error(msg);
            GP_ERR_COMMAND
        }
        Err(std_mpsc::RecvTimeoutError::Timeout) => {
            set_error("device did not answer");
            GP_ERR_COMMAND
        }
        Err(std_mpsc::RecvTimeoutError::Disconnected) => {
            set_error("connection closed");
            GP_ERR_COMMAND
        }
    }
}

/// Sets the simulated location. Blocks until the device acknowledged it.
#[unsafe(no_mangle)]
pub extern "C" fn gp_set_location(latitude: f64, longitude: f64) -> i32 {
    if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        set_error("coordinates out of range");
        return GP_ERR_ARGS;
    }
    send_command(|reply| Command::Set(latitude, longitude, reply))
}

/// Stops simulating; the device returns to its real GPS position.
#[unsafe(no_mangle)]
pub extern "C" fn gp_clear_location() -> i32 {
    send_command(Command::Clear)
}

/// Clears the location and closes the connection.
#[unsafe(no_mangle)]
pub extern "C" fn gp_disconnect() {
    drop_session();
    STATE.store(GP_STATE_IDLE, Ordering::SeqCst);
}

/// GP_STATE_IDLE, GP_STATE_CONNECTED or GP_STATE_LOST.
#[unsafe(no_mangle)]
pub extern "C" fn gp_state() -> i32 {
    STATE.load(Ordering::SeqCst)
}

/// Copies the last error (UTF-8, NUL-terminated, truncated) into `buf`.
///
/// # Safety
/// `buf` must point to at least `len` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gp_last_error(buf: *mut c_char, len: usize) {
    if buf.is_null() || len == 0 {
        return;
    }
    let msg = LAST_ERROR.lock().map(|e| e.clone()).unwrap_or_default();
    let mut end = msg.len().min(len - 1);
    while end > 0 && !msg.is_char_boundary(end) {
        end -= 1;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(msg.as_ptr(), buf as *mut u8, end);
        *buf.add(end) = 0;
    }
}

/// Returns 1 if `path` holds a readable RemotePairing file, else 0.
///
/// # Safety
/// `path` must be a valid NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gp_pairing_file_valid(path: *const c_char) -> i32 {
    let Some(path) = (unsafe { cstr(path) }) else {
        return 0;
    };
    match std::fs::read(path) {
        Ok(bytes) => RpPairingFile::from_bytes(&bytes).is_ok() as i32,
        Err(_) => 0,
    }
}
