//! La partie boutique du client : ses messages, pas son protocole.
//!
//! Le protocole — framing gRPC-Web, protobuf minimal, trailers, transport unaire
//! et flux serveur — vient de `this-client-web`, dans le framework. Ce qui reste
//! ici n'appartient qu'à cette boutique : le nom de `List` et la forme de sa
//! réponse. C'est la frontière : le framework porte ce que tout projet
//! réécrirait, la démo porte ce qu'aucun autre projet ne réécrira.

pub use this_client_web::{Message, call, subscribe};

/// `List` de cette boutique : le nom de l'entité en entrée, ses enregistrements
/// JSON en sortie.
pub async fn list(path: &str, entity: &str) -> Result<Vec<serde_json::Value>, String> {
    this_client_web::call_list(path, entity)
        .await?
        .iter()
        .map(|raw| serde_json::from_str(raw).map_err(|error| error.to_string()))
        .collect()
}
