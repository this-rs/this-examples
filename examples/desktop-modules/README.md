# desktop-modules — un domaine, trois surfaces

Une boutique e-commerce dont l'interface est **dérivée du descripteur d'un module this-rs**,
et dont le **même code** tourne en app desktop (Tauri), en app mobile (Tauri iOS/Android) et
en site web.

```
                    ┌──────────────────────────────┐
   module `shop` ──▶│  shop-core  (domaine, Rust)  │  aucun Tauri, aucun HTTP
   (descripteur +   │  ModuleStore · validation ·  │  → tests du domaine ici
    bundles fr/en)  │  stock · total · refus       │
                    └──────────┬───────────────────┘
                               │  appels de fonctions
              ┌────────────────┴────────────────┐
              ▼                                 ▼
   ┌────────────────────┐            ┌──────────────────────┐
   │ desktop-modules    │            │ shop-server          │
   │ Tauri 2 : lib+bin  │            │ axum : /api + dist   │
   │ desktop ET mobile  │            └──────────┬───────────┘
   └─────────┬──────────┘                       │ fetch
             │ invoke                            │
             └────────────────┬──────────────────┘
                              ▼
                    ┌──────────────────────────┐
                    │  ui/  (Leptos CSR, wasm) │  un seul bundle
                    │  api.rs choisit le       │  pour les trois
                    │  transport à l'exécution │  surfaces
                    └────────────┬─────────────┘
                                 │ dépend de
                 ┌───────────────┴────────────────┐
                 ▼                                ▼
      ┌────────────────────┐          ┌────────────────────────┐
      │ app-kit (pur)      │          │ ui-shell (Leptos)      │
      │ boot · settings ·  │          │ splash · panneau ·     │
      │ surface — 16 tests │          │ réglages · thème       │
      └────────────────────┘          └────────────────────────┘

   les capacités (qui peut quoi) ne sont PAS ici :
   elles viennent de `this_descriptor::Capabilities`, calculées
   depuis les politiques du descripteur et la session de la surface.
```

## Ce qui est partagé, ce qui change

| Morceau | Partagé ? |
|---|---|
| Domaine (stock, total, validation, erreurs) | **oui**, `shop-core`, un seul exemplaire |
| Contrats d'app (démarrage, réglages, accès) | **oui**, `app-kit`, testés sans navigateur |
| Composants d'app (splash, panneau, réglages) | **oui**, `ui-shell` |
| UI (écrans, formulaires générés) | **oui**, `ui/`, un seul crate wasm |
| Design | **oui**, une feuille ; la largeur décide sidebar ⇄ onglets bas |
| Transport | non : `invoke` (Tauri) ou `fetch` (`/api`) — `ui/src/api.rs` |
| Coquille | non : fenêtre native / navigateur / webview mobile |
| Stockage | non : répertoire de données de l'app / répertoire du serveur |

Le front lit `window.__TAURI_INTERNALS__` à l'exécution et affiche la surface détectée
(`desktop`, `mobile`, `web`) ; les réglages montrent aussi le transport et l'emplacement
des données.

## Les crates d'app, en couches

```
app-kit (pur)            boot · settings · surface                     16 tests
ui-shell (le kit)
  ├── model              modèle d'affichage d'un module (Kind typés)     5 tests
  ├── design             primitives : Tone typé, badge, jauge, tuile
  ├── browser            tableau d'entités (le kit décide *comment*)
  └── splash · panel · settings_panel · theme · storage · app_mode
ui (l'application)       écran de pilotage, formulaire de commande, câblage
```

**`model`** — plus d'interprétation de JSON dans l'interface : le descripteur typé du
framework est traduit **une fois** en un `Module` (entités, champs `Kind::Enum/Ref/Decimal`,
liens, langues). N'importe quel module produit le même modèle, donc les mêmes écrans.

**`design`** — les tons sont un enum, pas une chaîne : `Tone::Dangr` ne compile pas, alors
qu'une classe `"dangre"` produisait silencieusement un badge sans couleur.

**`browser`** — la séparation qui rend le tableau transposable :

| Qui | Décide |
|---|---|
| le kit | colonnes, alignement, en-tête collant, version « cartes » ≤ 700 px, action de ligne |
| l'app | le contenu d'une cellule (`Cell::Amount`, `Badge`, `Meter`, `Person`, `Reference`…) |

Le kit ignore ce qu'est un « stock » ou un « client » : l'app traduit ses conventions en
cellules, le kit les dessine.

## Le connecteur

Les neuf opérations sont décrites **une fois** dans `ui/src/api.rs`, les deux surfaces côte
à côte :

```rust
pub async fn save(entity: &str, id: Option<String>, data: Value) -> Result<Value, String> {
    send(Call::post(
        "save_record",                       // commande Tauri
        &format!("/records/{entity}"),       // route HTTP
        json!({ "entity": entity, "id": id, "data": data }),
        json!({ "id": id, "data": data }),
    ))
    .await
}
```

Un seul point de dispatch (`send`) dans tout le front. Ajouter une opération = une ligne ;
impossible d'en oublier une des deux moitiés.

## Appareil, permissions système, avis

Le kit distingue deux autorisations qu'on confond souvent :

| | Qui décide | Où c'est défini |
|---|---|---|
| **Capacités sur les données** | le module (politiques du descripteur) | `this_descriptor::Capabilities` |
| **Permissions système** (notifications, caméra…) | l'OS | `app_kit::Device` |

`app_kit::Device` est un trait : le kit **demande**, chaque coquille répond. `BrowserDevice`
(`ui-shell`) branche les notifications du navigateur et répond « non supporté » pour caméra,
micro, localisation et partage — plutôt que de proposer un bouton inerte. Les implémentations
natives viendront derrière le même trait, sans toucher au kit ni à l'app.

La demande est **asynchrone par contrat** (`on_result`) : une permission système passe par
une boîte de dialogue, un retour synchrone serait un mensonge.

`app_kit::Notices` tient la file d'avis sous trois règles : une clé = un avis (dix erreurs
identiques ne font pas dix toasts, la dernière remplace), au plus `max` vivants (le plus
ancien cède), expiration par l'**horloge de l'appelant** — donc testable à la milliseconde,
sans `sleep`. `ui_shell::ToastHost` la rend ; sans permission de notification, la coquille
dégrade vers le toast et ne perd pas le message.

Mesuré : demande d'accès → toast « Autorisation accordée : crm.delete. », disparu seul au
bout de six secondes (0 toast restant).

### Décidé, pas encore fait

- **gRPC par défaut**, côté front **et** templates : il faut `tonic-web` côté serveur et un
  client grpc-web en wasm (framing 5 octets + trailers à la main), le pincement d'`EventSource`.
- **QUIC / HTTP/3** : transport à ajouter côté serveur (`quinn` + `h3`), en plus de HTTP/1.1
  et HTTP/2 — c'est un protocole de transport, pas un protocole applicatif : il ne change rien
  au contrat, seulement la façon dont les octets passent.

## Transports

Le **front web** ne parle plus que gRPC-Web : les neuf opérations de `shop.data.v1.Data`
et le flux `shop.events.v1.Events/Subscribe`. Le serveur, lui, expose la même chose sur
cinq surfaces, pour les autres clients :

| Surface | Chemin | Pour qui |
|---|---|---|
| gRPC-Web | `/shop.data.v1.Data/*` · `/shop.events.v1.Events/Subscribe` | le front (même port qu'axum, donc pas de CORS) |
| REST | `/api/*` | scripts, curl, intégrations |
| SSE | `/api/events` | clients sans gRPC-Web |
| WebSocket | `/api/ws` | clients qui préfèrent un flux symétrique |
| gRPC | `:50051` | services internes |

La coquille native (desktop / iOS) garde `invoke` et les événements Tauri : elle n'a pas de
serveur HTTP à joindre.

**Deux pièges mesurés**, tous les deux invisibles à la compilation :

1. `tonic-web` répond à une erreur en *trailers-only* — `HTTP 200`, `grpc-status` dans les
   **en-têtes**, corps vide. Un client qui ne lit que les trailers du corps rapporte un
   succès sur un refus (l'interface affichait « Enregistrement supprimé » sans rien
   supprimer). Les deux emplacements doivent être lus.
2. Un RPC de streaming répond dans un **corps HTTP unique** : il faut lire le
   `ReadableStream` et découper les trames au fur et à mesure. Attendre la fin du corps,
   c'est attendre la fin d'un flux qui ne finit pas.

## Lancer

### Desktop (Tauri)

```sh
cd src-tauri
cargo tauri dev                            # trunk serve + fenêtre
cargo tauri build --debug --no-bundle      # binaire autonome → src-tauri/target/debug/
```

La coquille a **son propre workspace** : elle est exclue de `this-examples` parce que Tauri
exige GTK/WebKit sous Linux, ce qui ferait échouer `cargo build --workspace` en CI pour tout
le dépôt. Le domaine (`shop-core`), les contrats (`app-kit`) et le serveur restent, eux,
membres du workspace portable.

> Un binaire produit par `cargo build` **seul** reste en mode dev : Tauri charge `devUrl`
> (localhost:1420) et la fenêtre est blanche si `trunk serve` ne tourne pas.

### Site web

```sh
(cd ui && trunk build --release)                         # produit ui/dist (3,4 Mo)
SHOP_ADDR=127.0.0.1:8790 \
SHOP_DATA=/tmp/shop-web-data \
SHOP_DIST=$PWD/ui/dist \
  cargo run -p shop_server
# http://127.0.0.1:8790/ — dist et API sur la même origine (pas de CORS)
```

`SHOP_SESSION=anonymous` (défaut : `user`) restreint la surface à la lecture : le front
grise les écritures et le serveur répond 403.

`SHOP_ADDR=0.0.0.0:8790` sert le site au réseau local : c'est la façon la plus rapide de
voir l'app sur un vrai téléphone, sans outillage mobile.

### iOS (simulateur)

```sh
xcrun simctl boot "iPhone 16 Pro" && open -a Simulator
cd src-tauri
cargo tauri ios init --ci          # une fois : génère gen/apple
cargo tauri ios dev "iPhone 16 Pro"
```

Deux pièges déjà rencontrés :
1. **cible lib obligatoire** — Tauri mobile construit `--lib`. Le crate doit donc exposer
   `src/lib.rs` avec `#[cfg_attr(mobile, tauri::mobile_entry_point)] pub fn run()`, et
   `src/main.rs` ne fait qu'appeler `run()`. Un crate binaire seul échoue avec
   `no library targets found in package`.
2. **`xcodegen` a besoin d'un nom d'utilisateur** — lancé depuis un environnement sans
   `USER`, `cargo tauri ios init` échoue sur `Couldn't find current username`.

`cargo tauri ios dev` **n'accepte pas** `--ci` (contrairement à `ios init`) : c'est une
option globale, à placer avant le sous-commande.

### iPhone (appareil physique)

```sh
cd src-tauri
cargo tauri ios dev "iPhone de Théotime"     # nom exact : xcrun devicectl list devices
```

Prérequis, tous vérifiables d'avance :

```sh
security find-identity -v -p codesigning    # une identité « Apple Development » valide
xcrun devicectl list devices                # l'appareil doit être « available (paired) »
```

**Signature — la recette qui marche** (mesurée sur cette machine) :

Le piège est l'équipe. `security find-identity` affiche des certificats
« Apple Development: … (A3SZ7QJUY6) », mais leur `OU` est `YHCD3HX4Q9` — et c'est
`YHCD3HX4Q9` la seule équipe pour laquelle **Xcode a un compte connecté**. Signer avec
l'identifiant du `CN` échoue (`No Account for Team`), signer avec l'`OU` passe :

```sh
openssl x509 -in cert.pem -noout -subject      # CN=… (A3SZ7QJUY6)  OU=YHCD3HX4Q9
defaults read com.apple.dt.Xcode IDEProvisioningTeamByIdentifier   # teamID = YHCD3HX4Q9
```

1. `gen/apple/project.yml` → `DEVELOPMENT_TEAM: YHCD3HX4Q9` + `CODE_SIGN_STYLE: Automatic`,
   puis `cd gen/apple && xcodegen generate --spec project.yml`.
   (`bundle.iOS.developmentTeam` dans `tauri.conf.json` n'est **pas** recopié dans le projet.)
2. `cargo tauri ios dev "iPhone de Théotime"` → build, signature avec le profil wildcard
   `iOS Team Provisioning Profile: *` (l'appareil y est déjà enregistré), installation, lancement.

Une fois la signature en place, deux façons d'installer :

| | Commande | Dépend du Mac ensuite ? |
|---|---|---|
| Développement | `cargo tauri ios dev "<device>"` | **oui** (le front est servi par `trunk serve` sur le réseau) |
| Autonome | `cargo tauri ios build --debug` puis `devicectl device install app` | non (front embarqué) |

```sh
cargo tauri ios build --debug
cd gen/apple/build/arm64 && unzip -q "this modules.ipa" -d /tmp/ipa
xcrun devicectl device install  app --device <udid> "/tmp/ipa/Payload/this modules.app"
xcrun devicectl device process launch --device <udid> dev.thisrs.desktop-modules
```

Le conteneur de données se remplit avec `devicectl device copy to --domain-type
appDataContainer --domain-identifier dev.thisrs.desktop-modules --source <modules>
--source <records> --destination "Library/Application Support/dev.thisrs.desktop-modules"`.

Sur l'iPhone, deux réglages utilisateur peuvent être nécessaires la première fois :
*Mode développeur* (Réglages → Confidentialité et sécurité) et l'approbation du certificat
(Réglages → Général → VPN et gestion de l'appareil).

### Android

`cargo tauri android init` puis `cargo tauri android dev` — **non mesuré ici** : le SDK
Android n'est pas installé sur cette machine (`~/Library/Android/sdk` absent).

## Comportement : une app sur mobile, une page sur le web

Le même code sert les deux, mais il ne doit pas **se comporter** pareil :

| | Coquille native (desktop / mobile) | Site web |
|---|---|---|
| Zoom (pincement, double-tap) | verrouillé | intact (zoom d'accessibilité) |
| Sélection de texte sur les contrôles | désactivée | normale |
| Menu sur appui long, halo de tap, rebond | désactivés | normaux |
| Champs de saisie | 16 px (iOS zoome en dessous) | 14 px |
| Hauteur | `100dvh` | `100dvh` |

Mécanique : `ui_shell::behave_like_app(surface)` écrit `<html data-surface="desktop|mobile|web">`
et, **seulement si la surface est native**, réécrit la balise `viewport` en
`maximum-scale=1, user-scalable=no`. Le CSS cible `:root[data-surface="desktop"|"mobile"]` ;
le site garde donc le zoom, la sélection et les gestes d'une page.

`touch-action: manipulation` est appliqué partout : il supprime le double-tap et le délai de
300 ms sur les contrôles sans retirer le pincement.

## Installer sur l'écran d'accueil (sans signature)

Le site est une **PWA installable** : `manifest.webmanifest` (`display: standalone`) et les
métadonnées `apple-mobile-web-app-*`. Sur iPhone : Safari → Partager → *Sur l'écran d'accueil*.
Elle s'ouvre alors en plein écran, sans barre de navigateur — et `behave_like_app` la traite
comme une coquille (`navigator.standalone` sur iOS, `display-mode: standalone` ailleurs) :
zoom verrouillé, sélection désactivée sur les contrôles.

Mesuré (environnement simulé) :

| Contexte | `data-surface` | Zoom | Sélection |
|---|---|---|---|
| Onglet de navigateur | `web` | libre | normale (accessibilité) |
| `matchMedia('(display-mode: standalone)')` | `mobile` | verrouillé | désactivée |
| `navigator.standalone` (iOS) | `mobile` | verrouillé | désactivée |
| Webview Tauri | `desktop` / `mobile` | verrouillé | désactivée |

## L'API HTTP (identique aux commandes Tauri)

| Méthode | Route | Commande Tauri |
|---|---|---|
| GET | `/api/status` | `shop_status` |
| GET | `/api/session` | `app_session` |
| POST | `/api/load` | `load_shop_cmd` |
| GET | `/api/interface` | `shop_interface` |
| GET | `/api/records/{entity}` | `list_records` |
| POST | `/api/records/{entity}` | `save_record` |
| DELETE | `/api/records/{entity}/{id}` | `delete_record` |
| POST | `/api/orders` | `place_order_cmd` |
| POST | `/api/permissions/request` | `permission_request` |

Un refus métier revient avec le code et le message **du module** :
`422 {"code":"shop.product.out_of_stock","message":"Stock insuffisant pour Webcam Full HD : 0 disponible(s)."}`

## Tests

```sh
cargo test -p shop-core     # 10 tests : module, seed, stock, total, refus, références,
                            #             validation, garde de politique, demande d'autorisation
cargo test -p app-kit       # 16 tests : démarrage, réglages, surface

# le kit se teste sur l'hôte (pas besoin de navigateur pour le modèle)
(cd examples/desktop-modules/ui-shell && cargo test)   # 5 tests : modèle et formats

# dans le dépôt du framework :
cargo test -p this-descriptor capabilities   # 20 tests : politiques -> capacités de front,
                                             #   sessions (rôles, permissions), manques
```

Le domaine et les contrats d'app se testent sans Tauri, sans serveur, sans navigateur.

## Limites connues

- Les **libellés** de champs (`Prix`, `Client`…) sont un dictionnaire d'affichage côté UI ;
  les messages d'erreur, eux, viennent des bundles du module. L'UI est en français alors
  que le module livre `fr` et `en` : rendre la langue réglable demande de passer les
  libellés d'écran en données locale-keyed (prochaine étape, pas un contrôle factice).
- `read_only` est respecté par le formulaire et par le domaine, sans test dédié.
- Le déchargement d'un module n'est pas exposé dans cette app (voir l'exemple `vfs-module`).
- Sur mobile, la sélection de texte est désactivée sur les contrôles (comportement d'app) ;
  elle reste active dans les champs de saisie et sur les données (`code`).
- L'app native tourne sur l'iPhone 16 Pro Max apparié (build `debug`, signée avec le profil
  wildcard de l'équipe `YHCD3HX4Q9`). La capture d'écran du téléphone n'est pas automatisable
  ici : `idevicescreenshot` réclame le service `screenshotr`, absent sur cet iOS.
- Sur iOS, l'app installée démarre sur un store vide : il faut cliquer « Charger la
  boutique » (ou copier un répertoire de données existant dans le conteneur).
