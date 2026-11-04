//! # Client gRPC-Web en wasm
//!
//! Un navigateur ne parle pas gRPC (HTTP/2 binaire) : il parle **gRPC-Web**, qui est du
//! gRPC emballé dans HTTP/1.1. Ce module fait le minimum honnête :
//!
//! - encodage d'un message `ListRequest { string entity = 1 }` ;
//! - framing gRPC-Web : `[compression: 1][longueur: 4 big-endian][message]` ;
//! - décodage d'un `ListReply { repeated string records_json = 1 }`, en sautant le cadre
//!   de trailers (bit `0x80`).
//!
//! Ce n'est pas une bibliothèque gRPC générale : c'est le strict nécessaire pour les deux
//! messages du service `Data`, et c'est ce qui permet de vérifier que le transport marche
//! avant de généraliser.

use gloo_net::http::Request;

/// Encode un varint (comme le fait protobuf pour les longueurs).
fn varint(mut value: usize, out: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

/// Lit un varint et rend (valeur, octets consommés).
fn read_varint(bytes: &[u8], at: usize) -> Option<(usize, usize)> {
    let mut value = 0usize;
    let mut shift = 0;
    let mut read = 0;
    loop {
        let byte = *bytes.get(at + read)?;
        value |= ((byte & 0x7f) as usize) << shift;
        read += 1;
        if byte & 0x80 == 0 {
            return Some((value, read));
        }
        shift += 7;
        if shift > 28 {
            return None;
        }
    }
}

/// `ListRequest { entity }` → un message protobuf (champ 1, LEN).
fn encode_list_request(entity: &str) -> Vec<u8> {
    let mut message = vec![0x0a];
    varint(entity.len(), &mut message);
    message.extend_from_slice(entity.as_bytes());
    message
}

/// Emballe un message dans le framing gRPC-Web (non compressé).
pub fn frame(message: &[u8]) -> Vec<u8> {
    let mut framed = Vec::with_capacity(message.len() + 5);
    framed.push(0x00);
    framed.extend_from_slice(&(message.len() as u32).to_be_bytes());
    framed.extend_from_slice(message);
    framed
}

/// Extrait les messages d'une réponse gRPC-Web ; ignore le cadre de trailers.
pub fn unframe(bytes: &[u8]) -> Vec<&[u8]> {
    let mut messages = Vec::new();
    let mut at = 0;
    while at + 5 <= bytes.len() {
        let flags = bytes[at];
        let len = u32::from_be_bytes([bytes[at + 1], bytes[at + 2], bytes[at + 3], bytes[at + 4]])
            as usize;
        let start = at + 5;
        let end = start + len;
        if end > bytes.len() {
            break;
        }
        // 0x80 = cadre de trailers : ce ne sont pas des données
        if flags & 0x80 == 0 {
            messages.push(&bytes[start..end]);
        }
        at = end;
    }
    messages
}

/// `ListReply` → la liste des enregistrements JSON (champ 1 répété).
pub fn decode_list_reply(message: &[u8]) -> Vec<String> {
    let mut records = Vec::new();
    let mut at = 0;
    while at < message.len() {
        let Some((tag, tag_len)) = read_varint(message, at) else {
            break;
        };
        at += tag_len;
        let field = tag >> 3;
        let wire = tag & 0x07;
        match wire {
            // LEN : chaîne ou message imbriqué
            2 => {
                let Some((len, len_len)) = read_varint(message, at) else {
                    break;
                };
                at += len_len;
                let end = at + len;
                if end > message.len() {
                    break;
                }
                if field == 1 {
                    if let Ok(text) = std::str::from_utf8(&message[at..end]) {
                        records.push(text.to_string());
                    }
                }
                at = end;
            }
            // varint / 64 bits / 32 bits : on saute
            0 => {
                let Some((_, n)) = read_varint(message, at) else {
                    break;
                };
                at += n;
            }
            1 => at += 8,
            5 => at += 4,
            _ => break,
        }
    }
    records
}

/// Appelle `shop.data.v1.Data/List` en gRPC-Web et rend les enregistrements décodés.
pub async fn list(path: &str, entity: &str) -> Result<Vec<serde_json::Value>, String> {
    let body = frame(&encode_list_request(entity));
    let response = Request::post(path)
        .header("content-type", "application/grpc-web+proto")
        .header("x-grpc-web", "1")
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    let bytes = response.binary().await.map_err(|e| e.to_string())?;
    if !(200..300).contains(&status) {
        return Err(format!("gRPC-Web HTTP {status}"));
    }

    let messages = unframe(&bytes);
    let Some(first) = messages.first() else {
        return Err("réponse gRPC-Web vide".into());
    };
    decode_list_reply(first)
        .iter()
        .map(|raw| serde_json::from_str(raw).map_err(|e| e.to_string()))
        .collect()
}
