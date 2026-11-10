import MapKit
import SwiftUI

/// Map with one pin. Tapping the map moves the pin; `recenter` is a counter
/// the parent increments to make the map jump to the pin.
struct MapView: UIViewRepresentable {
    @Binding var latitude: Double?
    @Binding var longitude: Double?
    var recenter: Int

    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }

    func makeUIView(context: Context) -> MKMapView {
        let map = MKMapView()
        map.showsUserLocation = true
        map.showsCompass = true
        let france = CLLocationCoordinate2D(latitude: 46.8, longitude: 2.5)
        map.setRegion(MKCoordinateRegion(center: france, latitudinalMeters: 1_200_000, longitudinalMeters: 1_200_000), animated: false)
        let tap = UITapGestureRecognizer(target: context.coordinator, action: #selector(Coordinator.tapped(_:)))
        map.addGestureRecognizer(tap)
        return map
    }

    func updateUIView(_ map: MKMapView, context: Context) {
        context.coordinator.parent = self
        let coordinator = context.coordinator

        if let lat = latitude, let lon = longitude {
            let coordinate = CLLocationCoordinate2D(latitude: lat, longitude: lon)
            if let pin = coordinator.pin {
                pin.coordinate = coordinate
            } else {
                let pin = MKPointAnnotation()
                pin.coordinate = coordinate
                map.addAnnotation(pin)
                coordinator.pin = pin
            }
            if coordinator.lastRecenter != recenter {
                coordinator.lastRecenter = recenter
                map.setRegion(MKCoordinateRegion(center: coordinate, latitudinalMeters: 1500, longitudinalMeters: 1500), animated: true)
            }
        } else if let pin = coordinator.pin {
            map.removeAnnotation(pin)
            coordinator.pin = nil
        }
    }

    final class Coordinator: NSObject {
        var parent: MapView
        var pin: MKPointAnnotation?
        var lastRecenter = 0

        init(_ parent: MapView) {
            self.parent = parent
        }

        @objc func tapped(_ gesture: UITapGestureRecognizer) {
            guard let map = gesture.view as? MKMapView else { return }
            let point = gesture.location(in: map)
            let coordinate = map.convert(point, toCoordinateFrom: map)
            parent.latitude = coordinate.latitude
            parent.longitude = coordinate.longitude
        }
    }
}
