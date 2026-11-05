//! # `desktop_modules` — la coquille native (desktop **et** mobile)
//!
//! Ce crate ne contient **aucune logique métier** : chaque commande Tauri traduit un appel
//! du front en un appel à `shop_core`, et lui rend son résultat tel quel (même JSON que la
//! surface HTTP). La boutique elle-même vit dans `shop-core`, partagée avec `shop-server`.
//!
//! Tout est dans `run()` : un binaire desktop (`main.rs`) l'appelle, et iOS/Android
//! l'appellent par `#[tauri::mobile_entry_point]`. C'est ce qui permet à la même coquille
//! de servir les trois surfaces sans dupliquer une commande.

use serde::Serialize;
use serde_json::Value;
use shop_core::{
    AppError, Data, Interface, LineIn, Operation, PermissionDecision, PermissionRequest, Session,
    SessionKind, Sessions, Status,
};
use tauri::{Emitter, Manager, State};


// ---- Commandes : une par opération de la surface commune -----------------------------

#[tauri::command]
async fn module_status(d: State<'_, Data>) -> Result<Status, AppError> {
    Ok(shop_core::status(&d).await)
}

#[tauri::command]
async fn module_load(d: State<'_, Data>) -> Result<Status, AppError> {
    shop_core::load_and_report(&d).await
}

/// App native locale : la personne qui ouvre l'app est chez elle, avec les permissions
/// accordées depuis l'interface (liste transmise, donc décidable côté front).
#[tauri::command]
fn app_session(sessions: State<'_, Sessions>) -> Session {
    sessions.session()
}

/// La réponse à une demande d'autorisation : décision + session à jour.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestOutcome {
    decision: PermissionDecision,
    session: Session,
    message: String,
}

/// Une demande d'autorisation venue de l'interface.
///
/// La décision appartient au domaine (`Sessions::request`), qui n'accorde que ce que la
/// politique de l'opération nomme ; cette coquille ne fait que la transmettre.
#[tauri::command]
async fn permission_request(
    d: State<'_, Data>,
    sessions: State<'_, Sessions>,
    req: PermissionRequest,
) -> Result<RequestOutcome, AppError> {
    let (decision, message) = sessions.request(&d, &req).await?;
    Ok(RequestOutcome {
        decision,
        session: sessions.session(),
        message,
    })
}

#[tauri::command]
async fn module_interface(d: State<'_, Data>) -> Result<Interface, AppError> {
    shop_core::interface(&d).await
}

#[tauri::command]
fn list_records(d: State<'_, Data>, entity: String) -> Vec<Value> {
    shop_core::list_records(&d, &entity)
}

#[tauri::command]
async fn save_record(
    d: State<'_, Data>,
    sessions: State<'_, Sessions>,
    entity: String,
    id: Option<String>,
    data: Value,
) -> Result<Value, AppError> {
    let op = if id.is_some() {
        Operation::Update
    } else {
        Operation::Create
    };
    shop_core::guard(&d, &sessions.session(), &entity, op).await?;
    shop_core::save(&d, &entity, id, data).await
}

#[tauri::command]
async fn delete_record(
    d: State<'_, Data>,
    sessions: State<'_, Sessions>,
    entity: String,
    id: String,
) -> Result<(), AppError> {
    shop_core::guard(&d, &sessions.session(), &entity, Operation::Delete).await?;
    shop_core::remove(&d, &entity, &id).await
}

#[tauri::command]
async fn place_order_cmd(
    d: State<'_, Data>,
    sessions: State<'_, Sessions>,
    customer_id: String,
    lines: Vec<LineIn>,
    state: String,
) -> Result<Value, AppError> {
    shop_core::guard(&d, &sessions.session(), "order", Operation::Create).await?;
    shop_core::place_order(&d, &customer_id, &lines, &state).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
/// Une fenêtre inactive ne reçoit pas les mouvements de souris : AppKit ne les
/// envoie que si la fenêtre les accepte, et `wry` ne règle que
/// `acceptsFirstMouse:` (les clics). Sans ça, `:hover` ne s'applique jamais
/// quand l'app est en arrière-plan : les feux restent gris et le survol ne les
/// rallume pas. Mesuré dans `wry-0.57.1` : aucune occurrence de
/// `acceptsMouseMovedEvents`.
#[cfg(target_os = "macos")]
fn accept_mouse_moved(window: &tauri::WebviewWindow) {
    use objc2::msg_send;
    use objc2::runtime::{AnyObject, Bool};

    if let Ok(pointer) = window.ns_window() {
        let ns_window = pointer as *mut AnyObject;
        // SAFETY: `ns_window` vient de Tauri et désigne la NSWindow vivante de
        // cette fenêtre ; l'appel est un setter standard d'AppKit.
        unsafe {
            let _: () = msg_send![ns_window, setAcceptsMouseMovedEvents: Bool::YES];
            // On relit : un setter qui échoue en silence laisserait le survol
            // mort sans que rien ne le dise.
            let accepted: Bool = msg_send![ns_window, acceptsMouseMovedEvents];
            eprintln!("acceptsMouseMovedEvents = {}", accepted.as_bool());
        }
    }
}

/// La position de la souris, relevée pour que le survol marche même quand la
/// fenêtre n'est pas active.
///
/// Pourquoi un relevé et pas un événement : WebKit n'installe ses zones de suivi
/// que pour la fenêtre active (`activeInKeyWindow`), donc une fenêtre en
/// arrière-plan ne produit aucun survol CSS — et ce masque vit dans WKWebView,
/// hors de portée. `acceptsMouseMovedEvents` ne suffit pas, mesuré : il vaut
/// `true` et le survol ne vient toujours pas.
///
/// `NSEvent.mouseLocation` ne demande aucune permission. On compare la position
/// à la zone des feux (leur boîte, marges comprises) et on ne parle à l'app que
/// quand l'état change.
#[cfg(target_os = "macos")]
fn watch_pointer(window: tauri::WebviewWindow) {
    use objc2::msg_send;
    use objc2_foundation::NSPoint;

    std::thread::spawn(move || {
        let mut inside = false;
        loop {
            // Fenêtre active : le CSS fait son travail. On ne relève rien et on
            // dort longtemps — un fil qui interroge le système douze fois par
            // seconde pour ne rien en faire est un fil qui ne dort jamais.
            if window.is_focused().unwrap_or(true) {
                if inside {
                    inside = false;
                    let _ = tauri::Emitter::emit(&window, "shell://pointer", false);
                }
                std::thread::sleep(std::time::Duration::from_millis(600));
                continue;
            }

            std::thread::sleep(std::time::Duration::from_millis(80));

            let where_is_it = unsafe {
                let mouse: NSPoint = msg_send![objc2::class!(NSEvent), mouseLocation];
                local_pointer(&window, mouse.x, mouse.y)
            };

            let should = matches!(where_is_it, Some((x, y)) if (3.0..=75.0).contains(&x) && (3.0..=29.0).contains(&y));
            if should != inside {
                inside = should;
                let _ = tauri::Emitter::emit(&window, "shell://pointer", inside);
                match where_is_it {
                    Some((x, y)) => eprintln!("pointer over dots = {inside}  local {x:.0},{y:.0}"),
                    None => eprintln!("pointer over dots = {inside}  (position indisponible)"),
                }
            }
        }
    });
}

/// `NSEvent.mouseLocation` donne des points, origine en bas à gauche de l'écran
/// principal ; la fenêtre se décrit en pixels physiques, origine en haut à
/// gauche. On ramène tout en points et on rend la position locale — le calcul
/// est ainsi vérifiable en le lisant, pas en le supposant.
#[cfg(target_os = "macos")]
fn local_pointer(window: &tauri::WebviewWindow, mouse_x: f64, mouse_y: f64) -> Option<(f64, f64)> {
    let Ok(Some(primary)) = window.primary_monitor() else {
        return None;
    };
    let position = window.outer_position().ok()?;
    let scale = window.scale_factor().unwrap_or(1.0);
    if scale <= 0.0 {
        return None;
    }

    let screen_height = primary.size().height as f64 / primary.scale_factor();
    let from_top = screen_height - mouse_y;
    let local_x = mouse_x - position.x as f64 / scale;
    let local_y = from_top - position.y as f64 / scale;

    Some((local_x, local_y))
}

    tauri::Builder::default()
        // Fenêtre, Dock, habillage et reprise : la coquille est un plugin, une ligne.
        .plugin(this_shell_tauri::init(this_shell_tauri::ShellConfig::default()))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("main") {
                accept_mouse_moved(&window);
                watch_pointer(window);
            }

            let dir = app.path().app_data_dir()?;
            let data = Data::open(dir).map_err(|e| e.to_string())?;
            // Les changements du domaine deviennent des événements de la coquille : le front
            // les applique comme ceux reçus en SSE sur le web.
            let mut changes = data.changes();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(change) = changes.recv().await {
                    if let Err(error) = handle.emit("module://change", &change) {
                        eprintln!("émission du changement impossible : {error}");
                    }
                }
            });
            app.manage(data);
            // App locale : session utilisateur, permissions complètes (vide) et extensibles.
            app.manage(Sessions::new(
                SessionKind::User,
                Vec::new(),
                Vec::<String>::new(),
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            module_status,
            app_session,
            permission_request,
            module_load,
            module_interface,
            list_records,
            save_record,
            delete_record,
            place_order_cmd
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'app Tauri");
}
