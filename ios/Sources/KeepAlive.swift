import CoreLocation
import Foundation

/// The simulated position only holds while this app keeps its connection
/// open. iOS suspends apps in the background, so while a position is active
/// we keep a low-power location session running, which lets the app stay
/// alive (blue location indicator in the status bar).
final class KeepAlive: NSObject, CLLocationManagerDelegate {
    static let shared = KeepAlive()

    private let manager = CLLocationManager()
    private var wanted = false

    private override init() {
        super.init()
        manager.delegate = self
        manager.desiredAccuracy = kCLLocationAccuracyThreeKilometers
        manager.distanceFilter = CLLocationDistanceMax
        manager.pausesLocationUpdatesAutomatically = false
    }

    func requestPermission() {
        if manager.authorizationStatus == .notDetermined {
            manager.requestWhenInUseAuthorization()
        }
    }

    func start() {
        wanted = true
        apply()
    }

    func stop() {
        wanted = false
        manager.stopUpdatingLocation()
    }

    private func apply() {
        guard wanted else { return }
        switch manager.authorizationStatus {
        case .authorizedAlways, .authorizedWhenInUse:
            manager.allowsBackgroundLocationUpdates = true
            manager.showsBackgroundLocationIndicator = true
            manager.startUpdatingLocation()
        case .notDetermined:
            manager.requestWhenInUseAuthorization()
        default:
            break
        }
    }

    func locationManagerDidChangeAuthorization(_ manager: CLLocationManager) {
        apply()
    }

    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
    }

    func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
    }
}
