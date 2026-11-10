import Foundation
import MapKit

/// Address suggestions while typing, using Apple's own search (free, built in).
final class AddressSearch: NSObject, ObservableObject, MKLocalSearchCompleterDelegate {
    @Published var suggestions: [MKLocalSearchCompletion] = []

    private let completer = MKLocalSearchCompleter()

    override init() {
        super.init()
        completer.delegate = self
        completer.resultTypes = [.address, .pointOfInterest]
    }

    func update(query: String) {
        let trimmed = query.trimmingCharacters(in: .whitespaces)
        if trimmed.count < 3 {
            suggestions = []
            completer.cancel()
        } else {
            completer.queryFragment = trimmed
        }
    }

    func clear() {
        suggestions = []
        completer.cancel()
    }

    /// Resolves a suggestion into coordinates.
    func resolve(_ completion: MKLocalSearchCompletion, done: @escaping (CLLocationCoordinate2D?) -> Void) {
        let request = MKLocalSearch.Request(completion: completion)
        MKLocalSearch(request: request).start { response, _ in
            let coordinate = response?.mapItems.first?.placemark.coordinate
            DispatchQueue.main.async {
                done(coordinate)
            }
        }
    }

    func completerDidUpdateResults(_ completer: MKLocalSearchCompleter) {
        suggestions = Array(completer.results.prefix(6))
    }

    func completer(_ completer: MKLocalSearchCompleter, didFailWithError error: Error) {
        suggestions = []
    }
}
