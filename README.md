# Location Spoofer — application iPhone

Simulation de position GPS directement depuis l'iPhone (iOS 17.4+), sans PC branché.

**Pour installer : suis `TUTORIEL.md`.**

## Contenu

| Dossier | Rôle |
|---|---|
| `core/` | Moteur en Rust : appairage distant, tunnel, service de localisation simulée. Exposé à Swift par 7 fonctions C (`ios/Bridge.h`). |
| `ios/` | Application SwiftUI (carte, recherche d'adresse, réglages) et script de construction de l'`.ipa`. |
| `pairtool/` | `LocationSpooferPair.exe` : crée le fichier d'appairage depuis le PC et le copie dans l'app. |
| `.github/workflows/build.yml` | Compilation automatique sur GitHub (Mac pour l'app, Windows pour l'outil). |

## Fonctionnement

L'app se connecte à son propre iPhone comme le ferait un ordinateur, à travers le VPN local
LocalDevVPN (adresse 10.7.0.1) :

fichier d'appairage → RemotePairing → tunnel chiffré → RSD → instruments développeur → localisation simulée.

## État

- Le moteur Rust est compilé et ses cas d'erreur testés sous Linux (`cargo test -p spoofer_core`).
- Les sources Swift sont vérifiées syntaxiquement, mais n'ont pas été compilées (pas de Mac) ni
  exécutées sur un iPhone.
- L'image développeur (DDI) n'est pas montée par l'app : elle doit l'avoir été par l'application
  Windows depuis le dernier redémarrage de l'iPhone.

## Crédits et licences

- [idevice](https://github.com/jkcoxson/idevice) (MIT) — protocoles Apple en Rust. Ce projet l'utilise
  comme dépendance, sans modification. Sa politique sur l'IA (fichier `AI.md` du dépôt) s'applique à
  quiconque voudrait lui proposer des contributions.
- LocalDevVPN — VPN local, à installer depuis l'App Store.
- Code de ce projet : MIT.
