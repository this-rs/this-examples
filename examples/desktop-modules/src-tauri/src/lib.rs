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
    tauri::Builder::default()
        // Fenêtre, Dock, habillage et reprise : la coquille est un plugin, une ligne.
        .plugin(this_shell_tauri::init(this_shell_tauri::ShellConfig::default()))
        .setup(|app| {

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
