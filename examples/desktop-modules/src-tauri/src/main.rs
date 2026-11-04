//! Binaire desktop : il ne fait qu'appeler la coquille partagée `run()`.
//!
//! Le même `run()` est le point d'entrée iOS/Android (via `tauri::mobile_entry_point`),
//! ce qui impose une cible **lib** — Tauri mobile construit `--lib`, pas `--bin`.

// Empêche l'ouverture d'une console Windows en release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    desktop_modules_lib::run();
}
