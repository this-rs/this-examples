//! Front Leptos (CSR) de la boutique.
//!
//! Le même bundle sert les trois surfaces : app desktop Tauri, app mobile Tauri, site web.
//! `api.rs` choisit le transport à l'exécution ; le reste des écrans ne sait pas sur quoi
//! il tourne, et le design s'adapte à la largeur (barre latérale ⇄ onglets bas).

mod api;
mod format;
mod grpc_web;
mod icons;
mod lab;
mod shell;
mod window;
mod views;

use leptos::mount::mount_to_body;

fn main() {
    mount_to_body(shell::App);
}
