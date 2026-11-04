//! # `ui-shell` — les composants que toute app this-rs en Leptos refait
//!
//! Ils ne connaissent ni la boutique, ni le module : uniquement `app-kit` (les contrats
//! purs) et le DOM.
//!
//! - [`Splash`] — l'écran de démarrage : ce qui se charge, ce qui a échoué, un bouton
//!   pour réessayer. Présentation pure : la temporisation vit dans `app_kit::Boot`.
//! - [`behave_like_app`] — la coquille se comporte comme une app (pas de zoom, pas de
//!   sélection accidentelle, pas de rebond) ; le site, lui, reste une page.
//! - [`design`] — les primitives du design system (ton, pastille, jauge, tuile, vide).
//! - [`browser`] — le tableau d'entités : le kit décide comment, l'app décide quoi.
//! - [`model`] — le modèle d'affichage d'un module, dérivé du descripteur typé.
//! - [`store`] — le magasin d'enregistrements, une piste réactive par entité.
//! - [`Panel`] — la coquille d'un panneau latéral (voile, en-tête, corps, fermeture).
//! - [`SettingsPanel`] — apparence (thème, mouvement) + faits d'accès fournis par l'app.
//! - [`storage`] / [`theme`] — la persistance `localStorage` et l'application au `<html>`.

pub mod app_mode;
pub mod browser;
pub mod design;
pub mod model;
pub mod panel;
pub mod settings_panel;
pub mod splash;
pub mod storage;
pub mod store;
pub mod theme;

pub use app_mode::behave_like_app;
pub use browser::{Cell, Column, EntityTable, Row};
pub use design::{badge, empty, kpi, meter, Tone};
pub use model::{count, money_f, Entity, Field, Kind, Link, Module};
pub use panel::Panel;
pub use settings_panel::SettingsPanel;
pub use splash::Splash;
pub use storage::LocalStorage;
pub use store::Store;
pub use theme::{apply_settings, now_ms};
