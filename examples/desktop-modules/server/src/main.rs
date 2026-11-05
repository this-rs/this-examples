//! # `shop-server` — l'adaptateur HTTP
//!
//! Le même domaine que l'app desktop (`shop-core`), exposé aux navigateurs. Il ne contient
//! pas de logique métier : il traduit une requête HTTP en appel de domaine et rend le
//! résultat tel quel, avec le même JSON que les commandes Tauri.
//!
//! Deux choses vivent ici et nulle part ailleurs : la **politique de session** de cette
//! surface (`SHOP_SESSION`, `SHOP_ROLES`) et le fait qu'une demande d'autorisation soit
//! accordée automatiquement (c'est une démo — un vrai hôte la validerait par un humain).
//!
//! Le bundle Leptos (`ui/dist`, produit par `trunk build`) est servi sur `/`, donc le site
//! et l'API partagent la même origine : aucune configuration CORS.
//!
//! ```text
//! SHOP_ADDR=127.0.0.1:8080 SHOP_DATA=./data SHOP_DIST=../ui/dist \
//!   SHOP_SESSION=user SHOP_ROLES=accountant cargo run -p shop_server
//! ```

mod draft;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use shop_core::{
    AppError, Data, Interface, LineIn, Operation, PermissionDecision, PermissionRequest, Session,
    SessionKind, Sessions, Status,
};
use this_module_harness::drafter::Drafter;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::services::{ServeDir, ServeFile};

pub mod events {
    //! Types générés depuis `proto/events.proto`.
    tonic::include_proto!("shop.events.v1");
}

pub mod data {
    //! Types générés depuis `proto/data.proto`.
    tonic::include_proto!("shop.data.v1");
}

use data::data_server::{Data as DataApi, DataServer};

use events::events_server::{Events, EventsServer};
use events::{Change as ProtoChange, Op, SubscribeRequest};

/// Ce que cet hôte donne à ses requêtes : les données et la session de la surface.
#[derive(Clone)]
struct AppState {
    data: Arc<Data>,
    sessions: Arc<Sessions>,
}

/// Un refus du domaine, rendu avec le code et le message du module.
struct ApiError(AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        ApiError(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0.code.as_str() {
            code if code.ends_with("not_found") => StatusCode::NOT_FOUND,
            // Le contrat du module a refusé l'opération pour cette session.
            "this.policy.denied" => StatusCode::FORBIDDEN,
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        };
        (
            status,
            Json(json!({ "code": self.0.code, "message": self.0.message })),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrderRequest {
    customer_id: String,
    lines: Vec<LineIn>,
    state: String,
}

#[derive(Deserialize)]
struct SaveRequest {
    #[serde(default)]
    id: Option<String>,
    data: Value,
}

/// La réponse à une demande d'autorisation : la décision **et** la session à jour, pour que
/// le front recalcule ses capacités sans second aller-retour.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestOutcome {
    decision: PermissionDecision,
    session: Session,
    message: String,
}

/// La session de la surface : `SHOP_SESSION=anonymous|user|admin|service` et
/// `SHOP_ROLES=accountant,admin`.
///
/// Une vraie app lirait un jeton. Ici l'environnement suffit à démontrer que la décision
/// vient du **contrat du module**, pas de l'interface.
fn session_of_env() -> (SessionKind, Vec<String>) {
    let kind = match std::env::var("SHOP_SESSION")
        .unwrap_or_else(|_| "user".into())
        .trim()
        .to_lowercase()
        .as_str()
    {
        "anonymous" => SessionKind::Anonymous,
        "admin" => SessionKind::Admin,
        "service" => SessionKind::Service,
        _ => SessionKind::User,
    };
    let roles = std::env::var("SHOP_ROLES")
        .map(|raw| {
            raw.split(',')
                .map(|role| role.trim().to_string())
                .filter(|role| !role.is_empty())
                .collect()
        })
        .unwrap_or_default();
    (kind, roles)
}

// ---- Les mêmes opérations que les commandes Tauri -------------------------------------

/// Demande à un modèle d'écrire un module. La clé reste côté serveur : la page
/// ne reçoit que le programme, jamais le secret.
async fn api_draft(
    Json(request): Json<DraftRequest>,
) -> Result<Json<DraftReply>, (StatusCode, String)> {
    let completion = draft::ChatCompletion::from_env().map_err(bad_request)?;
    let drafter = Drafter::new(completion);
    match drafter.draft(&request.prompt).await {
        Ok(source) => Ok(Json(DraftReply { source })),
        Err(error) => Err(bad_request(error.to_string())),
    }
}

fn bad_request(message: String) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, message)
}

#[derive(serde::Deserialize)]
struct DraftRequest {
    prompt: String,
}

#[derive(serde::Serialize)]
struct DraftReply {
    source: String,
}

async fn api_status(State(st): State<AppState>) -> Json<Status> {
    Json(shop_core::status(&st.data).await)
}

async fn api_session(State(st): State<AppState>) -> Json<Session> {
    Json(st.sessions.session())
}

async fn api_load(State(st): State<AppState>) -> Result<Json<Status>, ApiError> {
    Ok(Json(shop_core::load_and_report(&st.data).await?))
}

async fn api_interface(State(st): State<AppState>) -> Result<Json<Interface>, ApiError> {
    Ok(Json(shop_core::interface(&st.data).await?))
}

async fn api_records(State(st): State<AppState>, Path(entity): Path<String>) -> Json<Vec<Value>> {
    Json(shop_core::list_records(&st.data, &entity))
}

async fn api_save(
    State(st): State<AppState>,
    Path(entity): Path<String>,
    Json(req): Json<SaveRequest>,
) -> Result<Json<Value>, ApiError> {
    let op = if req.id.is_some() {
        Operation::Update
    } else {
        Operation::Create
    };
    shop_core::guard(&st.data, &st.sessions.session(), &entity, op).await?;
    Ok(Json(
        shop_core::save(&st.data, &entity, req.id, req.data).await?,
    ))
}

async fn api_delete(
    State(st): State<AppState>,
    Path((entity, id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    shop_core::guard(&st.data, &st.sessions.session(), &entity, Operation::Delete).await?;
    shop_core::remove(&st.data, &entity, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn api_order(
    State(st): State<AppState>,
    Json(req): Json<OrderRequest>,
) -> Result<Json<Value>, ApiError> {
    shop_core::guard(&st.data, &st.sessions.session(), "order", Operation::Create).await?;
    Ok(Json(
        shop_core::place_order(&st.data, &req.customer_id, &req.lines, &req.state).await?,
    ))
}

/// Le fil de changements, en Server-Sent Events.
///
/// Une seconde fenêtre (ou un autre appareil) branchée ici corrige son magasin sans
/// interroger l'API : c'est le même `Change` que côté Tauri, seul le transport diffère.
async fn api_events(
    State(st): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = BroadcastStream::new(st.data.changes()).filter_map(|message| match message {
        Ok(change) => Event::default().json_data(&change).ok().map(Ok),
        // abonné en retard : on saute le changement plutôt que de fermer le flux
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Le même fil de changements, en WebSocket.
///
/// Utile aux clients qui n'ont pas d'`EventSource` (application native, autre langage) et
/// qui veulent une connexion symétrique : le client peut parler, on ignore ce qu'il dit —
/// cette surface est un flux sortant.
async fn api_ws(State(st): State<AppState>, upgrade: WebSocketUpgrade) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| ws_loop(socket, st.data.changes()))
}

async fn ws_loop(mut socket: WebSocket, mut changes: broadcast::Receiver<shop_core::Change>) {
    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                None | Some(Err(_)) | Some(Ok(Message::Close(_))) => break,
                // un client n'écrit pas dans ce flux : on ignore ses messages
                Some(Ok(_)) => {}
            },
            change = changes.recv() => match change {
                Ok(change) => {
                    let Ok(text) = serde_json::to_string(&change) else { continue };
                    if socket.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                // abonné en retard : on saute, la connexion reste
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            },
        }
    }
}

/// Le fil de changements en gRPC (server streaming).
///
/// Troisième protocole, même producteur : un service interne ou un autre langage s'abonne
/// ici sans parler HTTP. L'enregistrement voyage en JSON : le proto ne fige pas le schéma
/// des entités, qui appartient au module.
struct EventsService {
    data: Arc<Data>,
}

#[tonic::async_trait]
impl Events for EventsService {
    type SubscribeStream =
        Pin<Box<dyn tokio_stream::Stream<Item = Result<ProtoChange, tonic::Status>> + Send>>;

    async fn subscribe(
        &self,
        request: tonic::Request<SubscribeRequest>,
    ) -> Result<tonic::Response<Self::SubscribeStream>, tonic::Status> {
        let filter = request.into_inner().entity;
        let stream = BroadcastStream::new(self.data.changes()).filter_map(move |message| {
            let change = message.ok()?;
            if !filter.is_empty() && change.entity != filter {
                return None;
            }
            Some(Ok(into_proto(change)))
        });
        Ok(tonic::Response::new(Box::pin(stream)))
    }
}

fn into_proto(change: shop_core::Change) -> ProtoChange {
    ProtoChange {
        entity: change.entity,
        id: change.id,
        op: match change.op {
            shop_core::ChangeOp::Created => Op::Created as i32,
            shop_core::ChangeOp::Updated => Op::Updated as i32,
            shop_core::ChangeOp::Deleted => Op::Deleted as i32,
        },
        record_json: change
            .record
            .map(|record| record.to_string())
            .unwrap_or_default(),
    }
}

/// L'API de données en gRPC / gRPC-Web.
///
/// Un navigateur ne parle pas gRPC (HTTP/2 binaire) : `tonic-web` traduit le gRPC-Web
/// (HTTP/1.1) vers le même service. Le service est donc le même que pour un client natif —
/// seul le transport change.
struct DataService {
    data: Arc<Data>,
    sessions: Arc<Sessions>,
}

/// Une erreur du domaine devient un statut gRPC portant **le même JSON** que la surface
/// REST : le front n'a qu'un format d'erreur à connaître, quel que soit le transport.
fn to_status(error: shop_core::AppError) -> tonic::Status {
    let code = match error.code.as_str() {
        code if code.ends_with("not_found") => tonic::Code::NotFound,
        "this.policy.denied" => tonic::Code::PermissionDenied,
        _ => tonic::Code::InvalidArgument,
    };
    let payload = serde_json::to_string(&error).unwrap_or_else(|_| error.message.clone());
    tonic::Status::new(code, payload)
}

fn json_status(error: serde_json::Error) -> tonic::Status {
    tonic::Status::invalid_argument(error.to_string())
}

#[tonic::async_trait]
impl DataApi for DataService {
    async fn status(
        &self,
        _: tonic::Request<data::Empty>,
    ) -> Result<tonic::Response<data::JsonReply>, tonic::Status> {
        let status = shop_core::status(&self.data).await;
        Ok(tonic::Response::new(data::JsonReply {
            json: serde_json::to_string(&status).map_err(json_status)?,
        }))
    }

    async fn load(
        &self,
        _: tonic::Request<data::Empty>,
    ) -> Result<tonic::Response<data::JsonReply>, tonic::Status> {
        let status = shop_core::load_and_report(&self.data)
            .await
            .map_err(to_status)?;
        Ok(tonic::Response::new(data::JsonReply {
            json: serde_json::to_string(&status).map_err(json_status)?,
        }))
    }

    async fn interface(
        &self,
        _: tonic::Request<data::Empty>,
    ) -> Result<tonic::Response<data::JsonReply>, tonic::Status> {
        let interface = shop_core::interface(&self.data).await.map_err(to_status)?;
        Ok(tonic::Response::new(data::JsonReply {
            json: serde_json::to_string(&interface).map_err(json_status)?,
        }))
    }

    async fn session(
        &self,
        _: tonic::Request<data::Empty>,
    ) -> Result<tonic::Response<data::JsonReply>, tonic::Status> {
        let session = self.sessions.session();
        Ok(tonic::Response::new(data::JsonReply {
            json: serde_json::to_string(&session).map_err(json_status)?,
        }))
    }

    async fn list(
        &self,
        request: tonic::Request<data::ListRequest>,
    ) -> Result<tonic::Response<data::ListReply>, tonic::Status> {
        let entity = request.into_inner().entity;
        let records = shop_core::list_records(&self.data, &entity);
        Ok(tonic::Response::new(data::ListReply {
            records_json: records.iter().map(|record| record.to_string()).collect(),
        }))
    }

    async fn save(
        &self,
        request: tonic::Request<data::SaveRequest>,
    ) -> Result<tonic::Response<data::JsonReply>, tonic::Status> {
        let request = request.into_inner();
        let data: Value = serde_json::from_str(&request.data_json).map_err(json_status)?;
        let id = (!request.id.is_empty()).then_some(request.id);
        let op = if id.is_some() {
            Operation::Update
        } else {
            Operation::Create
        };
        shop_core::guard(&self.data, &self.sessions.session(), &request.entity, op)
            .await
            .map_err(to_status)?;
        let saved = shop_core::save(&self.data, &request.entity, id, data)
            .await
            .map_err(to_status)?;
        Ok(tonic::Response::new(data::JsonReply {
            json: saved.to_string(),
        }))
    }

    async fn remove(
        &self,
        request: tonic::Request<data::RemoveRequest>,
    ) -> Result<tonic::Response<data::Empty>, tonic::Status> {
        let request = request.into_inner();
        shop_core::guard(
            &self.data,
            &self.sessions.session(),
            &request.entity,
            Operation::Delete,
        )
        .await
        .map_err(to_status)?;
        shop_core::remove(&self.data, &request.entity, &request.id)
            .await
            .map_err(to_status)?;
        Ok(tonic::Response::new(data::Empty {}))
    }

    async fn place_order(
        &self,
        request: tonic::Request<data::PlaceOrderRequest>,
    ) -> Result<tonic::Response<data::JsonReply>, tonic::Status> {
        let request = request.into_inner();
        let lines: Vec<LineIn> = serde_json::from_str(&request.lines_json).map_err(json_status)?;
        shop_core::guard(
            &self.data,
            &self.sessions.session(),
            "order",
            Operation::Create,
        )
        .await
        .map_err(to_status)?;
        let order =
            shop_core::place_order(&self.data, &request.customer_id, &lines, &request.state)
                .await
                .map_err(to_status)?;
        Ok(tonic::Response::new(data::JsonReply {
            json: order.to_string(),
        }))
    }

    async fn request_permission(
        &self,
        request: tonic::Request<data::PermissionJson>,
    ) -> Result<tonic::Response<data::JsonReply>, tonic::Status> {
        let request: PermissionRequest =
            serde_json::from_str(&request.into_inner().request_json).map_err(json_status)?;
        let (decision, message) = self
            .sessions
            .request(&self.data, &request)
            .await
            .map_err(to_status)?;
        let outcome = serde_json::json!({
            "decision": decision,
            "session": self.sessions.session(),
            "message": message,
        });
        Ok(tonic::Response::new(data::JsonReply {
            json: outcome.to_string(),
        }))
    }
}

/// Une demande d'autorisation venue de l'interface.
///
/// Le domaine décide (il ne peut accorder que ce que la politique de l'opération nomme) ;
/// cette surface décide seulement *comment* : ici, accord immédiat pour la démo.
async fn api_permission_request(
    State(st): State<AppState>,
    Json(req): Json<PermissionRequest>,
) -> Result<Json<RequestOutcome>, ApiError> {
    let (decision, message) = st.sessions.request(&st.data, &req).await?;
    Ok(Json(RequestOutcome {
        decision,
        session: st.sessions.session(),
        message,
    }))
}

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt().with_target(false).init();

    let addr: SocketAddr = std::env::var("SHOP_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()
        .map_err(|e| format!("SHOP_ADDR invalide : {e}"))?;
    let data_dir = PathBuf::from(std::env::var("SHOP_DATA").unwrap_or_else(|_| "data".into()));
    let dist = PathBuf::from(std::env::var("SHOP_DIST").unwrap_or_else(|_| "../ui/dist".into()));

    let data = Data::open(data_dir.clone())?;
    let (kind, roles) = session_of_env();
    let sessions = Sessions::new(kind, roles, Vec::<String>::new());
    tracing::info!("session de la surface : {}", sessions.session().label());

    // Boutique de démonstration : si le store est vide, on charge le module et ses données,
    // sinon la première page du site serait un écran vide.
    if !shop_core::status(&data).await.loaded {
        shop_core::load_and_report(&data)
            .await
            .map_err(|e| format!("chargement du module : {}", e.message))?;
        tracing::info!("module shop chargé dans {}", data_dir.display());
    }

    let state = AppState {
        data: Arc::new(data),
        sessions: Arc::new(sessions),
    };
    // le service gRPC partage le même domaine que l'API REST
    let data_for_grpc = state.data.clone();
    let sessions_for_grpc = state.sessions.clone();
    let data_for_events = state.data.clone();

    // gRPC : même flux, autre protocole. Port séparé, et désactivable (`SHOP_GRPC_ADDR=off`).
    let grpc = std::env::var("SHOP_GRPC_ADDR").unwrap_or_else(|_| "127.0.0.1:50051".into());
    if grpc != "off" {
        let grpc_addr: SocketAddr = grpc
            .parse()
            .map_err(|e| format!("SHOP_GRPC_ADDR invalide : {e}"))?;
        let events = EventsServer::new(EventsService {
            data: state.data.clone(),
        });
        tokio::spawn(async move {
            tracing::info!("grpc://{grpc_addr}");
            if let Err(error) = tonic::transport::Server::builder()
                .add_service(events)
                .serve(grpc_addr)
                .await
            {
                tracing::error!("serveur gRPC : {error}");
            }
        });
    }

    let index = dist.join("index.html");
    let app = Router::new()
        .route("/api/status", get(api_status))
        .route("/api/session", get(api_session))
        .route("/api/load", post(api_load))
        .route("/api/interface", get(api_interface))
        .route("/api/records/{entity}", get(api_records).post(api_save))
        .route("/api/records/{entity}/{id}", delete(api_delete))
        .route("/api/orders", post(api_order))
        .route("/api/permissions/request", post(api_permission_request))
        .route("/api/draft", post(api_draft))
        .route("/api/events", get(api_events))
        .route("/api/ws", get(api_ws))
        .with_state(state);

    // gRPC et gRPC-Web sur le **même port** qu'axum : même origine, pas de CORS.
    // `tonic-web` est une couche : elle traduit le gRPC-Web (HTTP/1.1) vers le même service,
    // donc un navigateur et un client natif appellent exactement la même implémentation.
    let grpc = tonic::service::Routes::new(DataServer::new(DataService {
        data: data_for_grpc,
        sessions: sessions_for_grpc,
    }))
    // le flux d'événements passe par la même route : gRPC-Web sait faire le streaming
    .add_service(EventsServer::new(EventsService {
        data: data_for_events,
    }))
    .into_axum_router()
    .layer(tonic_web::GrpcWebLayer::new());

    let app = app
        .merge(grpc)
        .fallback_service(ServeDir::new(&dist).not_found_service(ServeFile::new(&index)));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("bind {addr} : {e}"))?;
    tracing::info!("http://{addr}  (dist : {})", dist.display());
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("serveur : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use shop_core::LineIn;

    #[tokio::test]
    async fn the_change_feed_is_served_over_grpc() {
        // un domaine isolé, pour ne pas dépendre d'un répertoire local
        let dir = std::env::temp_dir().join(format!("shop-grpc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let data = Arc::new(Data::open(dir.clone()).unwrap());
        shop_core::load_and_report(&data).await.unwrap();

        // le service sur un port éphémère
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let service = EventsServer::new(EventsService { data: data.clone() });
        tokio::spawn(async move {
            let _ = tonic::transport::Server::builder()
                .add_service(service)
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
                .await;
        });

        let mut client = events::events_client::EventsClient::connect(format!("http://{addr}"))
            .await
            .unwrap();
        let mut stream = client
            .subscribe(SubscribeRequest {
                entity: String::new(),
            })
            .await
            .unwrap()
            .into_inner();

        // une écriture quelconque
        shop_core::place_order(
            &data,
            "cust-camille",
            &[LineIn {
                product_id: "prod-casque".into(),
                qty: 1,
            }],
            "draft",
        )
        .await
        .unwrap();

        let change = tokio::time::timeout(std::time::Duration::from_secs(5), stream.message())
            .await
            .expect("un message dans les 5 s")
            .unwrap()
            .unwrap();
        assert_eq!(change.entity, "order");
        assert_eq!(change.op, Op::Created as i32);
        assert!(
            change.record_json.contains("number"),
            "{}",
            change.record_json
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
