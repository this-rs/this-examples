//! Démonstration lisible des quatre issues de la signature de modules.
//!
//! ```text
//! cargo run -p module_signing_example
//! ```
//!
//! Sortie non nulle si un scénario ne se comporte pas comme la règle l'exige : ce
//! binaire est utilisable comme test de fumée dans la CI.

use module_signing_example::run_all;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let outcomes = run_all().await;

    println!("{:<40} {:<10} code", "scénario", "résultat");
    println!("{}", "-".repeat(76));
    let mut all_ok = true;
    for o in &outcomes {
        let verdict = if o.accepted { "ACCEPTÉ" } else { "refusé" };
        let code = o.code.clone().unwrap_or_else(|| "—".to_string());
        println!("{:<40} {:<10} {}", o.label, verdict, code);
        if !o.as_expected() {
            all_ok = false;
            let wanted = if o.expected_accept {
                "accepté"
            } else {
                "refusé"
            };
            println!("  ⚠ inattendu : attendu {wanted}");
        }
    }
    println!("{}", "-".repeat(76));

    if all_ok {
        println!("OK — chaque scénario suit la règle de signature.");
    } else {
        eprintln!("ÉCHEC — au moins un scénario ne suit pas la règle.");
        std::process::exit(1);
    }
}
