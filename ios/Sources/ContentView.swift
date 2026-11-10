import MapKit
import SwiftUI
import UIKit
import UniformTypeIdentifiers

struct ContentView: View {
    @StateObject private var link = DeviceLink()
    @StateObject private var search = AddressSearch()
    @Environment(\.scenePhase) private var scenePhase

    @State private var latitude: Double?
    @State private var longitude: Double?
    @State private var recenter = 0
    @State private var query = ""
    @State private var latText = ""
    @State private var lonText = ""
    @State private var showImporter = false
    @State private var showHelp = false
    @State private var importFailed = false
    @FocusState private var searchFocused: Bool

    var body: some View {
        ZStack(alignment: .bottom) {
            MapView(latitude: $latitude, longitude: $longitude, recenter: recenter)
                .ignoresSafeArea()

            VStack(spacing: 10) {
                if link.hasPairingFile {
                    controls
                } else {
                    setup
                }
            }
            .padding(14)
            .background(.regularMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 20, style: .continuous))
            .padding(.horizontal, 10)
            .padding(.bottom, 6)
        }
        .onChange(of: latitude) { _ in syncFields() }
        .onChange(of: longitude) { _ in syncFields() }
        .onChange(of: query) { text in search.update(query: text) }
        .onChange(of: scenePhase) { phase in
            if phase == .active {
                link.refreshPairingFile()
                link.checkStillActive()
            }
        }
        .onChange(of: link.status) { status in
            if case .active = status {
                KeepAlive.shared.start()
            } else if case .working = status {
                // unchanged while an operation is running
            } else {
                KeepAlive.shared.stop()
            }
        }
        .onAppear { KeepAlive.shared.requestPermission() }
        .fileImporter(isPresented: $showImporter, allowedContentTypes: [UTType.item]) { result in
            if case .success(let url) = result {
                importFailed = !link.importPairingFile(from: url)
            }
        }
        .alert("Fichier refusé", isPresented: $importFailed) {
            Button("OK", role: .cancel) {}
        } message: {
            Text("Ce fichier n'est pas un fichier d'appairage valide. Utilise celui créé par LocationSpooferPair sur le PC.")
        }
        .sheet(isPresented: $showHelp) { HelpView() }
    }

    // MARK: - First launch: pairing file missing

    private var setup: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Configuration requise")
                .font(.headline)
            Text("1. Installe LocalDevVPN (App Store) et active-le.\n2. Sur le PC, iPhone branché en USB, lance LocationSpooferPair : il crée le fichier d'appairage et le copie ici.\n3. Reviens dans cette app.")
                .font(.subheadline)
                .foregroundColor(.secondary)
            HStack {
                Button("Importer un fichier…") { showImporter = true }
                    .buttonStyle(.bordered)
                Spacer()
                Button("Vérifier") { link.refreshPairingFile() }
                    .buttonStyle(.borderedProminent)
            }
            Button("Ouvrir LocalDevVPN") { openVPN() }
                .font(.subheadline)
        }
    }

    // MARK: - Main controls

    private var controls: some View {
        VStack(spacing: 10) {
            HStack {
                Image(systemName: "magnifyingglass").foregroundColor(.secondary)
                TextField("Adresse, ville, lieu…", text: $query)
                    .focused($searchFocused)
                    .autocorrectionDisabled(true)
                    .submitLabel(.search)
                    .onSubmit {
                        if let first = search.suggestions.first { pick(first) }
                    }
                if !query.isEmpty {
                    Button {
                        query = ""
                        search.clear()
                    } label: {
                        Image(systemName: "xmark.circle.fill").foregroundColor(.secondary)
                    }
                }
                Button { showHelp = true } label: { Image(systemName: "questionmark.circle") }
            }
            .padding(10)
            .background(Color(.secondarySystemBackground))
            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))

            if searchFocused && !search.suggestions.isEmpty {
                VStack(alignment: .leading, spacing: 0) {
                    ForEach(Array(search.suggestions.enumerated()), id: \.offset) { _, item in
                        Button { pick(item) } label: {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(item.title).font(.subheadline).foregroundColor(.primary)
                                if !item.subtitle.isEmpty {
                                    Text(item.subtitle).font(.caption).foregroundColor(.secondary)
                                }
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.vertical, 7)
                        }
                        Divider()
                    }
                }
            }

            HStack(spacing: 8) {
                coordinateField("Latitude", text: $latText)
                coordinateField("Longitude", text: $lonText)
            }

            HStack(spacing: 8) {
                Button { apply() } label: {
                    Text("Définir la position")
                        .fontWeight(.semibold)
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .disabled(isWorking)

                Button(role: .destructive) { link.reset() } label: {
                    Text("Reset Location").frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .disabled(isWorking)
            }

            statusLine
        }
    }

    private func coordinateField(_ title: String, text: Binding<String>) -> some View {
        TextField(title, text: text)
            .keyboardType(.numbersAndPunctuation)
            .autocorrectionDisabled(true)
            .padding(9)
            .background(Color(.secondarySystemBackground))
            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
            .onSubmit { useTypedCoordinates() }
    }

    private var statusLine: some View {
        VStack(alignment: .leading, spacing: 3) {
            HStack(spacing: 8) {
                switch link.status {
                case .idle:
                    Image(systemName: "circle").foregroundColor(.secondary)
                    Text("Position réelle").foregroundColor(.secondary)
                case .working(let text):
                    ProgressView()
                    Text(text).foregroundColor(.secondary)
                case .active(let lat, let lon):
                    Image(systemName: "location.fill").foregroundColor(.green)
                    Text(String(format: "Position simulée active : %.5f, %.5f", lat, lon))
                case .failed(let message):
                    Image(systemName: "exclamationmark.triangle.fill").foregroundColor(.orange)
                    Text(message)
                }
                Spacer(minLength: 0)
            }
            .font(.footnote)
            if !link.detail.isEmpty {
                Text(link.detail)
                    .font(.caption2)
                    .foregroundColor(.secondary)
                    .lineLimit(3)
            }
            if case .failed = link.status {
                Button("Ouvrir LocalDevVPN") { openVPN() }.font(.footnote)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private var isWorking: Bool {
        if case .working = link.status { return true }
        return false
    }

    // MARK: - Actions

    private func pick(_ item: MKLocalSearchCompletion) {
        searchFocused = false
        query = item.title
        search.clear()
        search.resolve(item) { coordinate in
            guard let coordinate = coordinate else { return }
            latitude = coordinate.latitude
            longitude = coordinate.longitude
            recenter += 1
        }
    }

    private func syncFields() {
        if let lat = latitude, let lon = longitude {
            latText = String(format: "%.6f", lat)
            lonText = String(format: "%.6f", lon)
        }
    }

    private func parse(_ text: String) -> Double? {
        Double(text.replacingOccurrences(of: ",", with: ".").trimmingCharacters(in: .whitespaces))
    }

    @discardableResult
    private func useTypedCoordinates() -> Bool {
        guard let lat = parse(latText), let lon = parse(lonText),
              (-90.0...90.0).contains(lat), (-180.0...180.0).contains(lon) else {
            return false
        }
        latitude = lat
        longitude = lon
        recenter += 1
        return true
    }

    private func apply() {
        searchFocused = false
        UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
        guard useTypedCoordinates(), let lat = latitude, let lon = longitude else {
            link.status = .failed("Choisis un point : recherche une adresse, touche la carte ou saisis des coordonnées valides.")
            return
        }
        link.setLocation(latitude: lat, longitude: lon)
    }

    private func openVPN() {
        guard let app = URL(string: "localdevvpn://enable"),
              let store = URL(string: "https://apps.apple.com/app/localdevvpn/id6755608044") else { return }
        UIApplication.shared.open(app, options: [:]) { opened in
            if !opened {
                DispatchQueue.main.async {
                    UIApplication.shared.open(store)
                }
            }
        }
    }
}

struct HelpView: View {
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List {
                Section("Utilisation") {
                    Text("1. Active LocalDevVPN.")
                    Text("2. Choisis un point : adresse, appui sur la carte ou coordonnées.")
                    Text("3. Touche « Définir la position ».")
                    Text("« Reset Location » rend le vrai GPS.")
                }
                Section("À savoir") {
                    Text("La position simulée tient tant que cette app reste ouverte (elle peut rester en arrière-plan). Si tu la fermes, le vrai GPS revient.")
                    Text("Après un redémarrage de l'iPhone, branche-le une fois au PC et clique sur « Définir la position » dans l'application Windows pour réactiver les services développeur.")
                    Text("Si tu changes ton code de déverrouillage, relance LocationSpooferPair sur le PC.")
                }
            }
            .navigationTitle("Aide")
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Fermer") { dismiss() }
                }
            }
        }
    }
}
