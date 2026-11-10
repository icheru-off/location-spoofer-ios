# Location Spoofer pour iPhone — tutoriel d'installation complet

Ce guide part de zéro et va jusqu'à l'app qui tourne sur ton iPhone, depuis un PC Windows.
Compte environ 1 heure la première fois (dont 20 minutes d'attente).

## Avant de commencer : ce qu'il faut savoir

**Ce que fait l'app.** Elle change la position GPS de l'iPhone directement depuis l'iPhone, sans PC branché : recherche d'adresse avec suggestions, appui sur la carte, coordonnées, bouton « Définir la position » et bouton « Reset Location ».

**État du projet.** Le moteur (la partie qui parle à l'iPhone) est compilé et testé sur mes machines pour ses cas d'erreur, mais **l'app n'a encore jamais tourné sur un vrai iPhone** : je n'ai ni Mac ni iPhone. Il est possible que la première compilation ou le premier lancement échoue. Dans ce cas, envoie-moi le message d'erreur (voir « Si ça ne marche pas » à la fin) et je corrige.

**Limites à connaître.**

- iOS 17.4 ou plus récent obligatoire.
- Avec un identifiant Apple gratuit, l'app expire au bout de **7 jours** : il faut la réinstaller avec Sideloadly (2 minutes, étape 5).
- Après chaque **redémarrage de l'iPhone**, il faut le brancher une fois au PC et cliquer sur « Définir la position » dans l'application Windows. C'est ce qui réactive les services développeur d'Apple ; ensuite l'app iPhone est autonome jusqu'au prochain redémarrage.
- La position simulée tient tant que l'app reste ouverte (elle peut passer en arrière-plan). Si tu la fermes, le vrai GPS revient.
- Le mode conduite n'est pas encore dans l'app iPhone : il reste dans l'application Windows. Je l'ajouterai une fois que la connexion de base fonctionne chez toi.

**Ce qu'il te faut.**

- Ton PC Windows 10 ou 11 et le câble USB de l'iPhone.
- Ton iPhone avec le Mode développeur déjà activé (c'est le cas, puisque l'application Windows fonctionne).
- Un identifiant Apple (le tien, ou un second créé pour l'occasion).
- Un compte GitHub gratuit (on le crée à l'étape 2).

---

## Étape 1 — Préparer le PC (une seule fois)

Sideloadly, l'outil qui installe l'app, exige les versions « site Apple » d'iTunes et d'iCloud. Les versions du Microsoft Store ne fonctionnent pas avec lui.

1. Ouvre **Paramètres Windows → Applications → Applications installées**.
2. Si tu vois **iTunes**, **iCloud** ou **Appareils Apple** installés depuis le Microsoft Store, désinstalle-les.
3. Télécharge et installe **iTunes** depuis le site d'Apple : https://www.apple.com/itunes/download/win64
4. Télécharge et installe **iCloud pour Windows** version site Apple (le lien direct est donné sur la page de téléchargement de Sideloadly, https://sideloadly.io).
5. Redémarre le PC.
6. Branche l'iPhone, déverrouille-le, ouvre iTunes une fois. Si l'iPhone demande « Faire confiance à cet ordinateur ? », touche **Se fier** et saisis ton code. Tu peux refermer iTunes.
7. Télécharge et installe **Sideloadly** : https://sideloadly.io (bouton Windows 64-bit).

L'application Windows Location Spoofer continue de fonctionner normalement avec cet iTunes.

---

## Étape 2 — Fabriquer l'app avec GitHub (20 minutes, dont 15 d'attente)

Une app iPhone ne peut être compilée que sur un Mac. GitHub prête gratuitement des Mac en ligne : tu lui donnes le code, il te rend l'app prête à installer.

### 2.1 Créer le compte

1. Va sur https://github.com et clique sur **Sign up**.
2. Saisis e-mail, mot de passe, nom d'utilisateur, puis valide le code reçu par e-mail.

### 2.2 Créer le dépôt

1. En haut à droite, clique sur **+** puis **New repository**.
2. **Repository name** : `location-spoofer-ios`.
3. Choisis **Public** (compilation gratuite et illimitée). Le code ne contient aucune information personnelle.
4. Ne coche rien d'autre. Clique sur **Create repository**.

### 2.3 Envoyer le code

1. Sur le PC, fais un clic droit sur `location-spoofer-iphone.zip` → **Extraire tout**.
2. Ouvre le dossier extrait `location-spoofer-iphone`. Tu dois voir : `.github`, `core`, `ios`, `pairtool`, `Cargo.toml`, `Cargo.lock`, etc.
3. Sur la page GitHub du dépôt, clique sur le lien **uploading an existing file**.
4. Dans l'explorateur Windows, sélectionne **tout le contenu** du dossier (Ctrl+A) et fais-le glisser dans la page GitHub. Attends la fin de l'envoi.
5. En bas, clique sur **Commit changes**.

**Vérification importante.** Sur la page du dépôt, tu dois voir un dossier `.github`. S'il manque (certains navigateurs ignorent les dossiers commençant par un point) :

1. Clique sur **Add file → Create new file**.
2. Dans le champ du nom, tape exactement : `.github/workflows/build.yml`
3. Ouvre sur le PC le fichier `workflow-build.yml.txt` avec le Bloc-notes, copie tout son contenu, colle-le dans la grande zone de texte GitHub.
4. Clique sur **Commit changes** (deux fois).

### 2.4 Attendre la compilation

1. Clique sur l'onglet **Actions** du dépôt.
2. Une ligne « Build » apparaît avec un rond jaune qui tourne. Compte 10 à 15 minutes.
3. Rond **vert** : c'est prêt. Croix **rouge** : la compilation a échoué — va à « Si ça ne marche pas ».

Si aucune ligne n'apparaît : onglet **Actions** → **Build** dans la colonne de gauche → **Run workflow** → **Run workflow**.

### 2.5 Télécharger les deux fichiers

1. Reviens sur la page d'accueil du dépôt (onglet **Code**).
2. Dans la colonne de droite, clique sur **Releases** puis sur « Build 1 ».
3. Télécharge :
   - `LocationSpoofer.ipa` — l'app iPhone ;
   - `LocationSpooferPair.exe` — le petit programme d'appairage pour le PC.

---

## Étape 3 — Installer LocalDevVPN sur l'iPhone

L'app a besoin de ce VPN local gratuit pour parler à son propre iPhone. Il ne fait sortir aucune donnée du téléphone.

1. Sur l'iPhone, ouvre l'App Store et installe **LocalDevVPN** (éditeur : Coxson Engineering LLC) : https://apps.apple.com/app/localdevvpn/id6755608044
2. Ouvre LocalDevVPN et touche **Connect**.
3. iOS demande d'ajouter une configuration VPN : touche **Autoriser** et saisis ton code.
4. L'état passe à « Connected » et une icône VPN apparaît en haut de l'écran.

---

## Étape 4 — Installer l'app sur l'iPhone avec Sideloadly

1. Branche l'iPhone au PC en USB et déverrouille-le.
2. Ouvre **Sideloadly**. Ton iPhone doit apparaître dans la liste **iDevice** en haut. S'il n'apparaît pas : rouvre iTunes une fois, vérifie « Se fier », change de câble ou de port USB.
3. Fais glisser `LocationSpoofer.ipa` sur l'icône **IPA** à gauche (ou clique dessus pour le choisir).
4. Dans **Apple ID**, saisis l'e-mail de ton identifiant Apple.
5. Ne touche pas aux options avancées. Clique sur **Start**.
6. Sideloadly demande le mot de passe de l'identifiant Apple, puis le code à 6 chiffres reçu sur ton iPhone ou un autre appareil Apple. Saisis-les.
7. Attends le message **Done.** en bas de la fenêtre.

À propos de l'identifiant Apple : Sideloadly l'utilise pour demander à Apple un certificat de développeur gratuit. Si tu préfères ne pas utiliser ton compte principal, crée un second identifiant sur https://account.apple.com et utilise celui-là.

### Autoriser l'app sur l'iPhone

L'icône « Location Spoofer » est maintenant sur l'écran d'accueil, mais iOS refuse de l'ouvrir tant que tu n'as pas approuvé le développeur :

1. **Réglages → Général → VPN et gestion de l'appareil**.
2. Sous **App du développeur**, touche ton identifiant Apple.
3. Touche **Faire confiance à « ton identifiant »**, puis **Se fier**.

---

## Étape 5 — Appairer l'app avec l'iPhone (une seule fois)

L'app a besoin d'un « fichier d'appairage » : une clé que l'iPhone accepte, créée depuis le PC.

1. Laisse l'iPhone branché en USB et **déverrouillé**.
2. Ferme l'application Windows Location Spoofer si elle est ouverte.
3. Double-clique sur `LocationSpooferPair.exe`.
4. Windows affiche peut-être « Windows a protégé votre ordinateur » (le programme n'est pas signé, c'est normal pour un outil que tu viens de compiler). Clique sur **Informations complémentaires** puis **Exécuter quand même**.
5. Une fenêtre noire s'ouvre. Regarde l'iPhone : si une demande de confiance apparaît, accepte-la et saisis ton code.
6. Résultat attendu dans la fenêtre :

```
iPhone detecte : 0000XXXX-XXXXXXXXXXXXXXXX
Fichier d'appairage cree : ...\rp_pairing_file.plist
Fichier copie dans l'app (...).
TERMINE. Ouvre Location Spoofer sur l'iPhone.
```

Le fichier `rp_pairing_file.plist` créé à côté du programme donne accès à ton iPhone : garde-le pour toi, ne l'envoie à personne.

### Transfert manuel (seulement si la copie automatique a échoué)

Si la fenêtre indique que le fichier n'a pas pu être copié dans l'app :

1. Ouvre **iTunes**, clique sur la petite icône d'iPhone en haut à gauche.
2. Dans la colonne de gauche, clique sur **Partage de fichiers**.
3. Clique sur **Location Spoofer** dans la liste des apps.
4. Fais glisser `rp_pairing_file.plist` dans la zone de droite (« Documents de Location Spoofer »).

Autre possibilité : envoie-toi le fichier (iCloud Drive par exemple), puis dans l'app touche **Importer un fichier…** et choisis-le.

---

## Étape 6 — Première utilisation

1. **Si l'iPhone a redémarré depuis ta dernière utilisation de l'application Windows** : branche-le au PC, ouvre l'application Windows, clique sur **Définir la position** une fois, puis **Reset Location**. Tu peux ensuite débrancher.
2. Sur l'iPhone, vérifie que **LocalDevVPN** est connecté.
3. Ouvre **Location Spoofer**.
4. Deux demandes d'autorisation apparaissent, à accepter :
   - localisation → **Autoriser lorsque l'app est active** ;
   - réseau local → **Autoriser**.
5. Si l'app affiche « Configuration requise », touche **Vérifier** (le fichier d'appairage vient d'être copié).
6. Choisis un point : tape une adresse et touche une suggestion, ou appuie sur la carte, ou saisis latitude et longitude.
7. Touche **Définir la position**. Après quelques secondes, la ligne du bas doit indiquer « Position simulée active ». Le point bleu de la carte se déplace à cet endroit.
8. Vérifie dans Plans ou une autre app.
9. **Reset Location** rend le vrai GPS.

Tant qu'une position est active, une pastille bleue de localisation reste affichée en haut de l'écran : c'est ce qui permet à l'app de rester en vie en arrière-plan.

---

## Tous les 7 jours : renouveler l'app

Quand l'app refuse de s'ouvrir (« n'est plus disponible ») :

1. Branche l'iPhone, ouvre Sideloadly.
2. Remets le même `LocationSpoofer.ipa`, le **même identifiant Apple**, clique sur **Start**.
3. Ne supprime pas l'app de l'iPhone avant : tu garderais ainsi le fichier d'appairage, sans refaire l'étape 5.

Un identifiant Apple gratuit permet 3 apps installées de cette façon en même temps.

---

## Dépannage

| Message dans l'app | Cause | Solution |
|---|---|---|
| « iPhone injoignable » | LocalDevVPN coupé | Ouvre LocalDevVPN, touche Connect, réessaie. Active aussi le Wi-Fi. |
| « L'iPhone a refusé le fichier d'appairage » | Fichier invalide (code de déverrouillage changé, iPhone restauré) | Refais l'étape 5. |
| « Fichier d'appairage absent ou illisible » | Étape 5 non faite, ou app supprimée puis réinstallée | Refais l'étape 5. |
| « Service développeur indisponible » | iPhone redémarré, ou Mode développeur désactivé | Étape 6, point 1. |
| « Le tunnel n'a pas pu s'ouvrir » | VPN dans un état instable | Coupe et réactive LocalDevVPN ; au besoin redémarre l'iPhone puis étape 6, point 1. |
| « La connexion a été interrompue » | iOS a fermé l'app en arrière-plan | Touche de nouveau « Définir la position ». |

| Problème ailleurs | Solution |
|---|---|
| Sideloadly ne voit pas l'iPhone | iTunes version site Apple installé ? iPhone déverrouillé ? « Se fier » accepté ? Autre câble. |
| Sideloadly : erreur de connexion Apple | Vérifie l'identifiant et le code à 6 chiffres ; réessaie quelques minutes plus tard. |
| L'app se ferme tout de suite à l'ouverture | Développeur non approuvé (fin de l'étape 4), ou app expirée (7 jours). |
| `LocationSpooferPair.exe` : « Service Apple introuvable » | iTunes version site Apple non installé (étape 1). |
| `LocationSpooferPair.exe` : « Appairage refuse » | iPhone verrouillé, ou iOS antérieur à 17.4. Déverrouille et relance. |

---

## Si ça ne marche pas : quoi m'envoyer

- **Croix rouge sur GitHub (étape 2.4)** : clique sur la ligne en échec, puis sur le bloc marqué d'une croix rouge, et copie les 30 dernières lignes du texte affiché.
- **Erreur de `LocationSpooferPair.exe`** : copie tout le texte de la fenêtre noire.
- **Erreur dans l'app iPhone** : fais une capture d'écran. Sous le message en français, une petite ligne grise en anglais donne le détail technique : c'est elle qui me permet de corriger.

Avec ça, je peux te renvoyer une version corrigée ; il suffira de remplacer les fichiers sur GitHub pour qu'il recompile.
