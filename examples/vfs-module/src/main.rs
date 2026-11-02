//! Démonstration lisible du cycle de vie d'un module stocké, en mémoire puis sur disque.
//!
//! ```text
//! cargo run -p vfs_module_example
//! ```
//!
//! Sortie non nulle si un cycle ne suit pas le contrat du store : utilisable comme
//! test de fumée.

use std::path::Path;

use vfs_module_example::{lifecycle_in_memory, lifecycle_on_disk, Lifecycle};

fn print(lc: &Lifecycle, title: &str) -> bool {
    println!("== {title} ==");
    println!(
        "  put 0.1.0      -> get(None) sert {}",
        lc.after_first_put.as_deref().unwrap_or("<rien>")
    );
    println!(
        "  put 0.2.0      -> get(None) sert {} (la plus haute)",
        lc.after_second_put.as_deref().unwrap_or("<rien>")
    );
    println!(
        "                    get(\"0.1.0\") sert {}",
        lc.explicit_old_version.as_deref().unwrap_or("<rien>")
    );
    let listed = lc
        .listed
        .iter()
        .map(|(n, v)| format!("{n}@{v}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!("  list()         -> [{listed}]");
    println!(
        "  remove 0.2.0   -> get(None) sert {}",
        lc.after_remove_new.as_deref().unwrap_or("<rien>")
    );
    println!(
        "  remove 0.1.0   -> get(None) echoue avec {}",
        lc.final_error.as_deref().unwrap_or("<aucune erreur>")
    );
    println!();
    lc.is_expected()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mem = lifecycle_in_memory().await;

    let dir = std::env::temp_dir().join(format!("vfs-module-demo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("creer le repertoire temporaire");
    let disk = lifecycle_on_disk(Path::new(&dir)).await;
    let _ = std::fs::remove_dir_all(&dir);

    let ok = print(&mem, "InMemoryVfs") & print(&disk, "LocalFsVfs");

    if ok {
        println!(
            "OK — les deux backends suivent le cycle attendu (charger, remplacer, decharger)."
        );
    } else {
        eprintln!("ECHEC — un cycle ne suit pas le contrat du store.");
        std::process::exit(1);
    }
}
