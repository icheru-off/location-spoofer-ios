import Foundation
import Network

/// Finds the TCP port of the phone's own RemotePairing service, which is
/// advertised over Bonjour as `_remotepairing._tcp`. Falls back to the usual
/// port when discovery gives nothing.
enum PortResolver {
    static let fallbackPort: UInt16 = 49152

    static func resolve(timeout: TimeInterval = 4) -> UInt16 {
        let search = Search()
        search.start()
        _ = search.done.wait(timeout: .now() + timeout)
        return search.finish() ?? fallbackPort
    }

    private final class Search {
        let done = DispatchSemaphore(value: 0)
        private let queue = DispatchQueue(label: "locationspoofer.port")
        private var browser: NWBrowser?
        private var connection: NWConnection?
        private var port: UInt16?
        private var finished = false
        private var pickedLoopback = false

        func start() {
            queue.async {
                let parameters = NWParameters.tcp
                parameters.includePeerToPeer = true
                let browser = NWBrowser(for: .bonjour(type: "_remotepairing._tcp", domain: "local."), using: parameters)
                self.browser = browser
                browser.browseResultsChangedHandler = { [weak self] results, _ in
                    self?.handle(results: results, parameters: parameters)
                }
                browser.stateUpdateHandler = { [weak self] state in
                    if case .failed = state {
                        self?.complete()
                    }
                }
                browser.start(queue: self.queue)
            }
        }

        /// Stops the search and returns the port found, if any.
        func finish() -> UInt16? {
            return queue.sync { () -> UInt16? in
                self.complete()
                return self.port
            }
        }

        private func handle(results: Set<NWBrowser.Result>, parameters: NWParameters) {
            if finished { return }
            // Prefer the service seen on the loopback interface: that is this phone.
            var chosen: NWBrowser.Result?
            var chosenIsLoopback = false
            for result in results {
                guard case .service = result.endpoint else { continue }
                let loopback = result.interfaces.contains { $0.name.hasPrefix("lo") }
                if chosen == nil || (loopback && !chosenIsLoopback) {
                    chosen = result
                    chosenIsLoopback = loopback
                }
            }
            guard let target = chosen else { return }
            if connection != nil && !(chosenIsLoopback && !pickedLoopback) { return }
            pickedLoopback = chosenIsLoopback

            connection?.cancel()
            let newConnection = NWConnection(to: target.endpoint, using: parameters)
            connection = newConnection
            newConnection.stateUpdateHandler = { [weak self, weak newConnection] state in
                guard let self = self, let current = newConnection else { return }
                switch state {
                case .ready, .waiting:
                    self.record(from: current)
                case .failed:
                    self.complete()
                default:
                    break
                }
            }
            newConnection.pathUpdateHandler = { [weak self, weak newConnection] path in
                guard let self = self, let current = newConnection else { return }
                if path.status == .satisfied {
                    self.record(from: current)
                }
            }
            newConnection.start(queue: queue)
        }

        private func record(from connection: NWConnection) {
            if finished { return }
            guard let endpoint = connection.currentPath?.remoteEndpoint else { return }
            if case .hostPort(_, let resolved) = endpoint, resolved.rawValue != 0 {
                port = resolved.rawValue
                complete()
            }
        }

        private func complete() {
            if finished { return }
            finished = true
            browser?.cancel()
            connection?.cancel()
            done.signal()
        }
    }
}
