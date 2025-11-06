use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use endpoint_benchmarks::{
    create_rest_server_in_memory, start_test_server, TestServer
};
use tokio::runtime::Runtime;
use serde_json::Value;

/// Structure pour les résultats d'un test de route imbriquée
#[derive(Debug)]
struct NestedRouteResult {
    route: String,
    depth: u8,
    response_time: u128, // en microseconds
    success: bool,
    entities_count: Option<usize>,
}

/// Fonction pour extraire un ID depuis une réponse JSON
async fn extract_first_id(response_body: &str) -> Option<String> {
    if let Ok(json) = serde_json::from_str::<Value>(response_body) {
        // Si c'est un array, prendre le premier élément
        if let Some(array) = json.as_array() {
            if let Some(first) = array.get(0) {
                return first.get("id")?.as_str().map(|s| s.to_string());
            }
        }
        // Si c'est un objet direct
        if let Some(id) = json.get("id") {
            return id.as_str().map(|s| s.to_string());
        }
        // Si c'est dans un wrapper "data"
        if let Some(data) = json.get("data") {
            if let Some(array) = data.as_array() {
                if let Some(first) = array.get(0) {
                    return first.get("id")?.as_str().map(|s| s.to_string());
                }
            }
        }
    }
    None
}

/// Fonction pour compter les entités dans une réponse
async fn count_entities(response_body: &str) -> Option<usize> {
    if let Ok(json) = serde_json::from_str::<Value>(response_body) {
        if let Some(array) = json.as_array() {
            return Some(array.len());
        }
        if let Some(data) = json.get("data") {
            if let Some(array) = data.as_array() {
                return Some(array.len());
            }
        }
        // Si c'est un objet unique
        if json.is_object() {
            return Some(1);
        }
    }
    None
}

/// Benchmark des routes imbriquées avec des IDs réels
async fn benchmark_nested_routes_with_real_ids(server: &TestServer) -> anyhow::Result<Vec<NestedRouteResult>> {
    let mut results = Vec::new();
    
    println!("\n🔗 === BENCHMARK ROUTES IMBRIQUÉES ===");
    
    // 1. Route de niveau 0 : /orders
    println!("📊 Niveau 0 : /orders");
    let start = std::time::Instant::now();
    let response = server.get("/orders").await?;
    let response_time = start.elapsed().as_micros();
    let body = TestServer::response_body_string(response).await?;
    let entities_count = count_entities(&body).await;
    
    results.push(NestedRouteResult {
        route: "/orders".to_string(),
        depth: 0,
        response_time,
        success: true,
        entities_count,
    });
    
    // Extraire un order_id pour les tests suivants
    let order_id = extract_first_id(&body).await;
    println!("   🆔 Order ID trouvé: {:?}", order_id);
    println!("   ⏱️  Temps: {}µs", response_time);
    println!("   📦 Entités: {:?}", entities_count);
    
    if let Some(order_id) = order_id {
        // 2. Route de niveau 1 : /orders/{id}
        println!("\n📊 Niveau 1 : /orders/{}", order_id);
        let route = format!("/orders/{}", order_id);
        let start = std::time::Instant::now();
        let response = server.get(&route).await?;
        let response_time = start.elapsed().as_micros();
        let body = TestServer::response_body_string(response).await?;
        let entities_count = count_entities(&body).await;
        
        results.push(NestedRouteResult {
            route: route.clone(),
            depth: 1,
            response_time,
            success: true,
            entities_count,
        });
        
        println!("   ⏱️  Temps: {}µs", response_time);
        println!("   📦 Entités: {:?}", entities_count);
        
        // 3. Route de niveau 2 : /orders/{id}/invoices
        println!("\n📊 Niveau 2 : /orders/{}/invoices", order_id);
        let route = format!("/orders/{}/invoices", order_id);
        let start = std::time::Instant::now();
        let response = server.get(&route).await?;
        let response_time = start.elapsed().as_micros();
        let body = TestServer::response_body_string(response).await?;
        let entities_count = count_entities(&body).await;
        
        results.push(NestedRouteResult {
            route: route.clone(),
            depth: 2,
            response_time,
            success: true,
            entities_count,
        });
        
        println!("   ⏱️  Temps: {}µs", response_time);
        println!("   📦 Entités: {:?}", entities_count);
        
        // Extraire un invoice_id pour les tests de niveau 3
        let invoice_id = extract_first_id(&body).await;
        println!("   🆔 Invoice ID trouvé: {:?}", invoice_id);
        
        if let Some(invoice_id) = invoice_id {
            // 4. Route de niveau 3 : /orders/{id}/invoices/{id}
            println!("\n📊 Niveau 3 : /orders/{}/invoices/{}", order_id, invoice_id);
            let route = format!("/orders/{}/invoices/{}", order_id, invoice_id);
            let start = std::time::Instant::now();
            let response = server.get(&route).await?;
            let response_time = start.elapsed().as_micros();
            let body = TestServer::response_body_string(response).await?;
            let entities_count = count_entities(&body).await;
            
            results.push(NestedRouteResult {
                route: route.clone(),
                depth: 3,
                response_time,
                success: true,
                entities_count,
            });
            
            println!("   ⏱️  Temps: {}µs", response_time);
            println!("   📦 Entités: {:?}", entities_count);
            
            // 5. Route de niveau 4 : /orders/{id}/invoices/{id}/payments
            println!("\n📊 Niveau 4 : /orders/{}/invoices/{}/payments", order_id, invoice_id);
            let route = format!("/orders/{}/invoices/{}/payments", order_id, invoice_id);
            let start = std::time::Instant::now();
            let response = server.get(&route).await?;
            let response_time = start.elapsed().as_micros();
            let body = TestServer::response_body_string(response).await?;
            let entities_count = count_entities(&body).await;
            
            results.push(NestedRouteResult {
                route: route.clone(),
                depth: 4,
                response_time,
                success: true,
                entities_count,
            });
            
            println!("   ⏱️  Temps: {}µs", response_time);
            println!("   📦 Entités: {:?}", entities_count);
            
            // Extraire un payment_id pour le test ultime
            let payment_id = extract_first_id(&body).await;
            println!("   🆔 Payment ID trouvé: {:?}", payment_id);
            
            if let Some(payment_id) = payment_id {
                // 6. Route de niveau 5 : /orders/{id}/invoices/{id}/payments/{id}
                println!("\n📊 Niveau 5 : /orders/{}/invoices/{}/payments/{}", order_id, invoice_id, payment_id);
                let route = format!("/orders/{}/invoices/{}/payments/{}", order_id, invoice_id, payment_id);
                let start = std::time::Instant::now();
                let response = server.get(&route).await?;
                let response_time = start.elapsed().as_micros();
                let body = TestServer::response_body_string(response).await?;
                let entities_count = count_entities(&body).await;
                
                results.push(NestedRouteResult {
                    route: route.clone(),
                    depth: 5,
                    response_time,
                    success: true,
                    entities_count,
                });
                
                println!("   ⏱️  Temps: {}µs", response_time);
                println!("   📦 Entités: {:?}", entities_count);
            }
        }
    }
    
    // Afficher le résumé
    println!("\n📈 === RÉSUMÉ PERFORMANCE PAR NIVEAU ===");
    for result in &results {
        let overhead = if result.depth == 0 { 0 } else { 
            result.response_time - results[0].response_time 
        };
        println!("Niveau {} | {}µs | Overhead: {}µs | Route: {}", 
                result.depth, result.response_time, overhead, result.route);
    }
    
    Ok(results)
}

fn benchmark_nested_routes_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Créer le serveur avec les données de test
    let server = rt.block_on(async {
        let router = create_rest_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("nested_routes_performance");

    // Benchmark du parcours complet des routes imbriquées
    group.bench_function("full_nested_route_traversal", |b| {
        b.to_async(&rt).iter(|| async {
            let results = benchmark_nested_routes_with_real_ids(black_box(&server)).await.unwrap();
            black_box(results)
        });
    });

    group.finish();
}

fn benchmark_nested_routes_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let server = rt.block_on(async {
        let router = create_rest_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let mut group = c.benchmark_group("nested_routes_scaling");

    // Obtenir des IDs pour les tests
    let (order_id, invoice_id, payment_id) = rt.block_on(async {
        // Récupérer des IDs réels
        let response = server.get("/orders").await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        let order_id = extract_first_id(&body).await.unwrap();
        
        let response = server.get(&format!("/orders/{}/invoices", order_id)).await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        let invoice_id = extract_first_id(&body).await.unwrap();
        
        let response = server.get(&format!("/orders/{}/invoices/{}/payments", order_id, invoice_id)).await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        let payment_id = extract_first_id(&body).await.unwrap();
        
        (order_id, invoice_id, payment_id)
    });

    // Benchmark de chaque niveau individuellement
    let routes = vec![
        ("level_0_orders", "/orders".to_string(), 0u8),
        ("level_1_order_by_id", format!("/orders/{}", order_id), 1u8),
        ("level_2_order_invoices", format!("/orders/{}/invoices", order_id), 2u8),
        ("level_3_specific_invoice", format!("/orders/{}/invoices/{}", order_id, invoice_id), 3u8),
        ("level_4_invoice_payments", format!("/orders/{}/invoices/{}/payments", order_id, invoice_id), 4u8),
        ("level_5_specific_payment", format!("/orders/{}/invoices/{}/payments/{}", order_id, invoice_id, payment_id), 5u8),
    ];

    for (name, route, level) in routes {
        group.bench_with_input(
            BenchmarkId::new("route_level", format!("{}_{}", level, name)),
            &(&server, route.as_str()),
            |b, (server, route)| {
                b.to_async(&rt).iter(|| async {
                    let start = std::time::Instant::now();
                    let response = server.get(route).await.unwrap();
                    let _body = TestServer::response_body_string(response).await.unwrap();
                    let response_time = start.elapsed().as_micros();
                    black_box(response_time)
                });
            },
        );
    }

    group.finish();
}

fn benchmark_nested_routes_concurrent(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let server = rt.block_on(async {
        let router = create_rest_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    // Obtenir des IDs réels
    let (order_id, invoice_id, payment_id) = rt.block_on(async {
        let response = server.get("/orders").await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        let order_id = extract_first_id(&body).await.unwrap();
        
        let response = server.get(&format!("/orders/{}/invoices", order_id)).await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        let invoice_id = extract_first_id(&body).await.unwrap();
        
        let response = server.get(&format!("/orders/{}/invoices/{}/payments", order_id, invoice_id)).await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        let payment_id = extract_first_id(&body).await.unwrap();
        
        (order_id, invoice_id, payment_id)
    });

    let mut group = c.benchmark_group("nested_routes_concurrent");

    // Test de concurrence sur différents niveaux
    let concurrent_levels = [10, 25, 50];
    
    for concurrent_req in concurrent_levels.iter() {
        // Test niveau 2 (modérément complexe)
        let route_level2 = format!("/orders/{}/invoices", order_id);
        group.bench_with_input(
            BenchmarkId::new("concurrent_level2", concurrent_req),
            &(route_level2.clone(), *concurrent_req),
            |b, (route, concurrent_req)| {
                let server = server.clone();
                b.to_async(&rt).iter(move || {
                    let server = server.clone();
                    let route = route.clone();
                    async move {
                        let tasks = (0..*concurrent_req).map(|_| {
                            let server = server.clone();
                            let route = route.clone();
                            tokio::spawn(async move {
                                let start = std::time::Instant::now();
                                let response = server.get(&route).await.unwrap();
                                let _body = TestServer::response_body_string(response).await.unwrap();
                                start.elapsed().as_micros()
                            })
                        });
                        
                        let results = futures::future::join_all(tasks).await;
                        let latencies: Vec<u128> = results.into_iter().map(|r| r.unwrap()).collect();
                        
                        black_box(latencies)
                    }
                });
            },
        );

        // Test niveau 5 (très complexe) 
        let route_level5 = format!("/orders/{}/invoices/{}/payments/{}", order_id, invoice_id, payment_id);
        group.bench_with_input(
            BenchmarkId::new("concurrent_level5", concurrent_req),
            &(route_level5.clone(), *concurrent_req),
            |b, (route, concurrent_req)| {
                let server = server.clone();
                b.to_async(&rt).iter(move || {
                    let server = server.clone();
                    let route = route.clone();
                    async move {
                        let tasks = (0..*concurrent_req).map(|_| {
                            let server = server.clone();
                            let route = route.clone();
                            tokio::spawn(async move {
                                let start = std::time::Instant::now();
                                let response = server.get(&route).await.unwrap();
                                let _body = TestServer::response_body_string(response).await.unwrap();
                                start.elapsed().as_micros()
                            })
                        });
                        
                        let results = futures::future::join_all(tasks).await;
                        let latencies: Vec<u128> = results.into_iter().map(|r| r.unwrap()).collect();
                        
                        black_box(latencies)
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_nested_routes_performance,
    benchmark_nested_routes_scaling,
    benchmark_nested_routes_concurrent
);

criterion_main!(benches);