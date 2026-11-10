//! LocationSpooferPair — run once on the PC with the iPhone plugged in (USB).
//!
//! 1. Pairs with the iPhone (RemotePairing, the scheme used by iOS 17.4+).
//! 2. Saves the resulting pairing file next to this program.
//! 3. Copies it into the Location Spoofer app on the iPhone, so the app can
//!    later talk to the phone on its own (through LocalDevVPN).

use std::io::Write;
use std::path::PathBuf;

use idevice::afc::opcode::AfcFopenMode;
use idevice::core_device_proxy::CoreDeviceProxy;
use idevice::house_arrest::HouseArrestClient;
use idevice::installation_proxy::InstallationProxyClient;
use idevice::provider::IdeviceProvider;
use idevice::remote_pairing::{RemotePairingClient, RpPairingFile};
use idevice::rsd::RsdHandshake;
use idevice::usbmuxd::{Connection, UsbmuxdAddr, UsbmuxdConnection};
use idevice::{IdeviceService, RemoteXpcClient};

const APP_EXECUTABLE: &str = "LocationSpoofer";
const FILE_NAME: &str = "rp_pairing_file.plist";
const HOST_NAME: &str = "LocationSpoofer";

fn pause() {
    print!("\nAppuie sur Entree pour fermer...");
    let _ = std::io::stdout().flush();
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
}

#[tokio::main]
async fn main() {
    println!("=== Location Spoofer : appairage de l'iPhone ===\n");
    let code = match run().await {
        Ok(()) => 0,
        Err(msg) => {
            println!("\n[ERREUR] {msg}");
            1
        }
    };
    pause();
    std::process::exit(code);
}

async fn run() -> Result<(), String> {
    // ---- 1. find the iPhone over USB ---------------------------------------
    let mut usbmuxd = UsbmuxdConnection::default().await.map_err(|e| {
        format!(
            "Service Apple introuvable ({e:?}).\n\
             Installe l'app « Appareils Apple » (Microsoft Store) ou iTunes, puis relance."
        )
    })?;
    let devices = usbmuxd
        .get_devices()
        .await
        .map_err(|e| format!("Impossible de lister les appareils : {e:?}"))?;
    let device = devices
        .iter()
        .find(|d| d.connection_type == Connection::Usb)
        .ok_or("Aucun iPhone detecte en USB. Branche-le, deverrouille-le, touche « Se fier ».")?;
    println!("iPhone detecte : {}", device.udid);
    let provider = device.to_provider(UsbmuxdAddr::default(), HOST_NAME);

    // ---- 2. pair -----------------------------------------------------------
    println!("\nAppairage en cours...");
    println!(">>> Regarde l'iPhone : s'il affiche une demande de confiance, accepte-la");
    println!(">>> et saisis ton code. Garde l'ecran deverrouille.\n");
    let pairing = pair(&provider).await.map_err(|e| {
        format!(
            "Appairage refuse : {e}\n\
             Verifie : iPhone deverrouille, iOS 17.4 ou plus recent, Mode developpeur active."
        )
    })?;
    let bytes = pairing.to_bytes();

    let local: PathBuf = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(FILE_NAME)))
        .unwrap_or_else(|| PathBuf::from(FILE_NAME));
    tokio::fs::write(&local, &bytes)
        .await
        .map_err(|e| format!("Impossible d'ecrire {} : {e}", local.display()))?;
    println!("Fichier d'appairage cree : {}", local.display());

    // ---- 3. copy it into the app -------------------------------------------
    match push_to_app(&provider, &bytes).await {
        Ok(bundle_id) => {
            println!("Fichier copie dans l'app ({bundle_id}).");
            println!("\nTERMINE. Ouvre Location Spoofer sur l'iPhone.");
        }
        Err(e) => {
            println!("\nLe fichier n'a pas pu etre copie automatiquement dans l'app :\n  {e}");
            println!("\nCopie-le a la main (voir le tutoriel, etape « Transfert manuel ») :");
            println!("  {}", local.display());
        }
    }
    println!("\nGarde ce fichier pour toi : il donne acces a ton iPhone.");
    Ok(())
}

/// USB -> CoreDeviceProxy tunnel -> RSD -> tunnel service -> RemotePairing.
async fn pair(provider: &dyn IdeviceProvider) -> Result<RpPairingFile, String> {
    let dbg = |what: &str, e: &dyn std::fmt::Debug| format!("{what} ({e:?})");

    let proxy = CoreDeviceProxy::connect(provider)
        .await
        .map_err(|e| dbg("connexion CoreDevice", &e))?;
    let rsd_port = proxy.tunnel_info().server_rsd_port;
    let adapter = proxy
        .create_software_tunnel()
        .map_err(|e| dbg("tunnel logiciel", &e))?;
    let mut adapter = adapter.to_async_handle();
    let rsd_stream = adapter
        .connect(rsd_port)
        .await
        .map_err(|e| dbg("connexion RSD", &e))?;
    let handshake = RsdHandshake::new(rsd_stream)
        .await
        .map_err(|e| dbg("poignee de main RSD", &e))?;

    let service = handshake
        .services
        .get("com.apple.internal.dt.coredevice.untrusted.tunnelservice")
        .ok_or("service de tunnel absent sur cet iPhone")?;
    let stream = adapter
        .connect(service.port)
        .await
        .map_err(|e| dbg("connexion au service de tunnel", &e))?;
    let mut conn = RemoteXpcClient::new(stream)
        .await
        .map_err(|e| dbg("RemoteXPC", &e))?;
    conn.do_handshake().await.map_err(|e| dbg("RemoteXPC", &e))?;
    let _ = conn.recv_root().await.map_err(|e| dbg("RemoteXPC", &e))?;

    let mut file = RpPairingFile::generate(HOST_NAME);
    let mut client = RemotePairingClient::new(conn, HOST_NAME);
    client
        .connect(&mut file, async || "000000".to_string())
        .await
        .map_err(|e| dbg("appairage", &e))?;
    Ok(file)
}

/// Finds the installed app (sideloading tools change its bundle id) and
/// writes the pairing file into its Documents folder.
async fn push_to_app(provider: &dyn IdeviceProvider, bytes: &[u8]) -> Result<String, String> {
    let mut proxy = InstallationProxyClient::connect(provider)
        .await
        .map_err(|e| format!("liste des apps indisponible ({e:?})"))?;
    let apps = proxy
        .get_apps(Some("User"), None)
        .await
        .map_err(|e| format!("liste des apps indisponible ({e:?})"))?;
    let bundle_id = apps
        .iter()
        .find(|(_, info)| {
            info.as_dictionary()
                .and_then(|d| d.get("CFBundleExecutable"))
                .and_then(|v| v.as_string())
                == Some(APP_EXECUTABLE)
        })
        .map(|(id, _)| id.clone())
        .ok_or("l'app Location Spoofer n'est pas installee sur l'iPhone (installe-la d'abord)")?;

    let house = HouseArrestClient::connect(provider)
        .await
        .map_err(|e| format!("acces aux documents de l'app refuse ({e:?})"))?;
    let mut afc = house
        .vend_documents(bundle_id.clone())
        .await
        .map_err(|e| format!("acces aux documents de l'app refuse ({e:?})"))?;
    let mut file = afc
        .open(format!("/Documents/{FILE_NAME}"), AfcFopenMode::WrOnly)
        .await
        .map_err(|e| format!("ecriture impossible ({e:?})"))?;
    file.write_entire(bytes)
        .await
        .map_err(|e| format!("ecriture impossible ({e:?})"))?;
    file.close()
        .await
        .map_err(|e| format!("ecriture impossible ({e:?})"))?;
    Ok(bundle_id)
}
