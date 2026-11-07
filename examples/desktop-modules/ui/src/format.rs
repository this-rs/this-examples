//! Libellés lisibles et lecture d'un enregistrement.
//!
//! Les **formats** (montants, milliers) vivent dans le kit : une seule définition pour toutes
//! les apps. Ici on ne garde que ce qui est propre à cette boutique : ses libellés.
//!
//! Depuis le passage au modèle typé, on ne lit plus le JSON du descripteur : un champ est un
//! [`Field`] et sa nature est un [`Kind`]. Le compilateur vérifie ce qu'une `Value` laissait
//! passer silencieusement.

use serde_json::Value;
use this_ui::{Entity, Field, Kind, Tone};

use crate::api::Snapshot;

pub use this_ui::{count as count_f, money_f};

/// Le vocabulaire de la boutique, quand l'entité en fait partie.
///
/// `None` signifie « entité inconnue » — un module drafté, par exemple. C'est
/// volontairement distinct d'un repli : sans cette distinction, le code ne peut
/// pas savoir s'il a un vrai libellé ou le nom brut.
pub fn known_label(name: &str) -> Option<&'static str> {
    Some(match name {
        "customer" => "Clients",
        "product" => "Produits",
        "order" => "Commandes",
        "order_item" => "Lignes de commande",
        "shipment" => "Expéditions",
        _ => return None,
    })
}

/// Met en forme un identifiant : `order_item` → « Order item ».
///
/// Un module drafté n'embarque pas encore de bundles de messages ; sans ça, la
/// navigation afficherait `order_item` tel quel, ce qui est un identifiant, pas un
/// libellé.
pub fn prettify(identifier: &str) -> String {
    let spaced = identifier.replace('_', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub fn entity_label(name: &str) -> String {
    known_label(name)
        .map(str::to_string)
        .unwrap_or_else(|| prettify(name))
}

pub fn entity_label_one(name: &str) -> String {
    match name {
        "customer" => "Client",
        "product" => "Produit",
        "order" => "Commande",
        "order_item" => "Ligne de commande",
        "shipment" => "Expédition",
        other => other,
    }
    .into()
}

pub fn field_label(name: &str) -> String {
    match name {
        "name" => "Nom",
        "email" => "Courriel",
        "city" => "Ville",
        "sku" => "Référence",
        "category" => "Catégorie",
        "price" => "Prix",
        "stock" => "Stock",
        "active" => "Actif",
        "number" => "Numéro",
        "customer" => "Client",
        "state" => "Statut",
        "total" => "Total",
        "placed_on" => "Date",
        "order" => "Commande",
        "product" => "Produit",
        "qty" => "Quantité",
        "unit_price" => "Prix unitaire",
        "carrier" => "Transporteur",
        "tracking" => "Suivi",
        other => other,
    }
    .into()
}

pub fn enum_label(value: &str) -> String {
    match value {
        "draft" => "Brouillon",
        "paid" => "Payée",
        "shipped" => "Expédiée",
        "cancelled" => "Annulée",
        "audio" => "Audio",
        "peripherique" => "Périphérique",
        "ecran" => "Écran",
        "accessoire" => "Accessoire",
        "maison" => "Maison",
        other => other,
    }
    .into()
}

/// Ton d'un état : la couleur double le texte, elle ne le remplace pas.
pub fn state_tone(state: &str) -> Tone {
    match state {
        "paid" => Tone::Ok,
        "shipped" => Tone::Info,
        "cancelled" => Tone::Danger,
        _ => Tone::Muted,
    }
}

pub fn num(value: &Value) -> f64 {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0.0)
}

pub fn money(value: &Value) -> String {
    money_f(num(value))
}

/// Texte d'une valeur pour l'affichage.
pub fn text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(true) => "Oui".into(),
        Value::Bool(false) => "Non".into(),
        Value::Null => "—".into(),
        other => other.to_string(),
    }
}

/// Valeur d'un enregistrement vers la chaîne éditée dans un formulaire.
pub fn to_form(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

pub fn find(snap: &Snapshot, entity: &str, id: &str) -> Option<Value> {
    snap.get(entity)?
        .iter()
        .find(|record| record["id"].as_str() == Some(id))
        .cloned()
}

/// Libellé lisible d'un enregistrement (nom, numéro, sinon identifiant).
pub fn display_name(entity: &str, record: &Value) -> String {
    let key = match entity {
        "customer" | "product" => "name",
        "order" => "number",
        _ => "id",
    };
    record[key].as_str().unwrap_or_default().to_string()
}

pub fn ref_label(snap: &Snapshot, entity: &str, id: &str) -> String {
    find(snap, entity, id)
        .map(|record| display_name(entity, &record))
        .unwrap_or_else(|| id.to_string())
}

/// Le texte d'un champ, selon sa **nature typée**.
pub fn cell(snap: &Snapshot, field: &Field, value: &Value) -> String {
    match &field.kind {
        Kind::Ref(entity) => ref_label(snap, entity, value.as_str().unwrap_or_default()),
        Kind::Decimal => money(value),
        Kind::Enum(_) => enum_label(value.as_str().unwrap_or_default()),
        Kind::Bool => {
            if value == &Value::Bool(true) {
                "Oui".into()
            } else {
                "Non".into()
            }
        }
        _ => text(value),
    }
}

/// Une entité est créable à la main si tous ses champs obligatoires sont saisissables.
pub fn creatable(entity: &Entity) -> bool {
    entity.creatable_by_hand()
}

/// Initiales d'un client, pour la pastille des tableaux.
pub fn initials(name: &str) -> String {
    let mut out = String::new();
    for part in name.split_whitespace().take(2) {
        if let Some(c) = part.chars().next() {
            out.extend(c.to_uppercase());
        }
    }
    if out.is_empty() {
        "?".into()
    } else {
        out
    }
}
