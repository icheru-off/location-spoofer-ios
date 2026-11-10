import Foundation
import SwiftUI

/// Talks to the iPhone itself through the Rust core (see Bridge.h).
/// All device work happens on a private serial queue; published state is
/// only touched on the main thread.
final class DeviceLink: ObservableObject {
    enum Status: Equatable {
        case idle
        case working(String)
        case active(Double, Double)
        case failed(String)
    }

    @Published var status: Status = .idle
    @Published var hasPairingFile = false
    @Published var detail = ""          // technical detail of the last error

    static let targetIP = "10.7.0.1"    // address given to the phone by LocalDevVPN
    static let pairingFileName = "rp_pairing_file.plist"

    private let queue = DispatchQueue(label: "locationspoofer.device")

    static var pairingFileURL: URL {
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        return docs.appendingPathComponent(pairingFileName)
    }

    init() {
        refreshPairingFile()
    }

    func refreshPairingFile() {
        let path = DeviceLink.pairingFileURL.path
        hasPairingFile = FileManager.default.fileExists(atPath: path) && gp_pairing_file_valid(path) == 1
    }

    /// Copies a pairing file picked in the Files app into the app's Documents.
    func importPairingFile(from url: URL) -> Bool {
        let scoped = url.startAccessingSecurityScopedResource()
        defer {
            if scoped { url.stopAccessingSecurityScopedResource() }
        }
        guard let data = try? Data(contentsOf: url) else { return false }
        do {
            try data.write(to: DeviceLink.pairingFileURL, options: .atomic)
        } catch {
            return false
        }
        refreshPairingFile()
        return hasPairingFile
    }

    func setLocation(latitude: Double, longitude: Double) {
        publish(.working("Connexion à l'iPhone…"))
        queue.async {
            var code = self.ensureConnected()
            if code == GP_OK {
                code = gp_set_location(latitude, longitude)
                if code == GP_ERR_COMMAND || code == GP_ERR_NOT_CONNECTED {
                    // The link dropped (app was suspended, VPN restarted…): retry once.
                    gp_disconnect()
                    code = self.ensureConnected()
                    if code == GP_OK {
                        code = gp_set_location(latitude, longitude)
                    }
                }
            }
            if code == GP_OK {
                self.publish(.active(latitude, longitude))
            } else {
                self.publishFailure(code)
            }
        }
    }

    func reset() {
        publish(.working("Réinitialisation…"))
        queue.async {
            if gp_state() == GP_STATE_CONNECTED {
                _ = gp_clear_location()
            }
            gp_disconnect()
            self.publish(.idle)
        }
    }

    /// Called when the app returns to the foreground: report a dropped link.
    func checkStillActive() {
        guard case .active = status else { return }
        queue.async {
            if gp_state() != GP_STATE_CONNECTED {
                self.publish(.failed("La connexion a été interrompue : la position simulée n'est plus active. Touche « Définir la position » pour la rétablir."))
            }
        }
    }

    // MARK: - Private

    private func ensureConnected() -> Int32 {
        if gp_state() == GP_STATE_CONNECTED {
            return GP_OK
        }
        let path = DeviceLink.pairingFileURL.path
        let discovered = PortResolver.resolve()
        var code = gp_connect(path, DeviceLink.targetIP, discovered)
        if code == GP_ERR_UNREACHABLE && discovered != PortResolver.fallbackPort {
            code = gp_connect(path, DeviceLink.targetIP, PortResolver.fallbackPort)
        }
        return code
    }

    private func lastError() -> String {
        var buffer = [CChar](repeating: 0, count: 512)
        gp_last_error(&buffer, buffer.count)
        return String(cString: buffer)
    }

    private func publish(_ newStatus: Status) {
        DispatchQueue.main.async {
            self.status = newStatus
            if case .failed = newStatus {
                // keep detail
            } else {
                self.detail = ""
            }
        }
    }

    private func publishFailure(_ code: Int32) {
        let technical = lastError()
        let message: String
        switch code {
        case GP_ERR_PAIRING_FILE:
            message = "Fichier d'appairage absent ou illisible. Relance LocationSpooferPair sur le PC."
        case GP_ERR_UNREACHABLE, GP_ERR_TIMEOUT:
            message = "iPhone injoignable. Active LocalDevVPN (et le Wi-Fi), puis réessaie."
        case GP_ERR_PAIRING_REJECTED:
            message = "L'iPhone a refusé le fichier d'appairage. Relance LocationSpooferPair sur le PC (à refaire si tu as changé ton code de déverrouillage)."
        case GP_ERR_TUNNEL:
            message = "Le tunnel n'a pas pu s'ouvrir. Coupe puis réactive LocalDevVPN et réessaie."
        case GP_ERR_DVT:
            message = "Service développeur indisponible. Vérifie que le Mode développeur est activé, puis branche l'iPhone au PC et clique une fois sur « Définir la position » dans l'application Windows (à refaire après chaque redémarrage de l'iPhone)."
        default:
            message = "La connexion a été perdue. Réessaie."
        }
        DispatchQueue.main.async {
            self.detail = technical
            self.status = .failed(message)
        }
    }
}
