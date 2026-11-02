//! Prouve que c'est la signature — et non l'écriture — qui autorise le chargement.

use module_signing_example::{
    codes, forged_signature_is_refused, signed_module_is_accepted, unknown_key_id_is_refused,
    unsigned_module_is_refused,
};

#[tokio::test]
async fn a_signed_module_loads_under_its_anchor() {
    let o = signed_module_is_accepted().await;
    assert!(
        o.accepted,
        "le module signé doit être accepté, code={:?}",
        o.code
    );
}

#[tokio::test]
async fn a_signature_forged_under_a_known_key_id_is_refused() {
    let o = forged_signature_is_refused().await;
    assert!(!o.accepted, "une signature forgée doit être refusée");
    assert_eq!(o.code.as_deref(), Some(codes::MODULE_BAD_SIGNATURE));
}

#[tokio::test]
async fn an_unknown_key_id_is_refused() {
    let o = unknown_key_id_is_refused().await;
    assert!(!o.accepted, "une clé inconnue doit être refusée");
    assert_eq!(o.code.as_deref(), Some(codes::MODULE_UNTRUSTED_KEY));
}

#[tokio::test]
async fn an_unsigned_module_is_refused_once_an_anchor_is_declared() {
    let o = unsigned_module_is_refused().await;
    assert!(!o.accepted, "un module non signé doit être refusé");
    assert_eq!(o.code.as_deref(), Some(codes::MODULE_UNSIGNED));
}
