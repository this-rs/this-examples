use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use endpoint_benchmarks::{create_rest_server_in_memory, start_test_server, TestServer};
use tokio::runtime::Runtime;

/// Benchmark simple des routes imbriquées avec IDs fixes
fn benchmark_nested_routes_simple(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let server = rt.block_on(async {
        let router = create_rest_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    // Obtenir des IDs réels d'abord
    let (order_id, invoice_id) = rt.block_on(async {
        // Récupérer la liste des orders
        let response = server.get("/orders").await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        println!("📋 Orders response: {}", body);
        
        // Parser pour extraire l'ID
        let orders: serde_json::Value = serde_json::from_str(&body).unwrap();
        let order_id = orders[0]["id"].as_str().unwrap().to_string();
        println!("🆔 Using Order ID: {}", order_id);
        
        // Récupérer les invoices de cet order
        let route = format!("/orders/{}/invoices", order_id);
        let response = server.get(&route).await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        println!("📋 Invoices response: {}", body);
        
        let invoices: serde_json::Value = serde_json::from_str(&body).unwrap();
        let invoice_id = invoices["data"][0]["target"]["id"].as_str().unwrap().to_string();
        println!("🆔 Using Invoice ID: {}", invoice_id);
        
        (order_id, invoice_id)
    });

    let mut group = c.benchmark_group("nested_routes_simple");

    // Test chaque niveau de route
    let routes = vec![
        ("level_0_orders", "/orders".to_string()),
        ("level_1_order_by_id", format!("/orders/{}", order_id)),
        ("level_2_order_invoices", format!("/orders/{}/invoices", order_id)),
        ("level_3_specific_invoice", format!("/orders/{}/invoices/{}", order_id, invoice_id)),
    ];

    for (name, route) in routes {
        group.bench_with_input(
            BenchmarkId::new("route", name),
            &route,
            |b, route| {
                let server = server.clone();
                b.to_async(&rt).iter(|| {
                    let server = server.clone();
                    let route = route.clone();
                    async move {
                        let start = std::time::Instant::now();
                        let response = server.get(&route).await.unwrap();
                        let _body = TestServer::response_body_string(response).await.unwrap();
                        let response_time = start.elapsed().as_micros();
                        
                        // Print stats for analysis
                        let depth = if route == "/orders" { 0 } 
                        else if route.contains("/invoices/") { 3 }
                        else if route.contains("/invoices") { 2 }
                        else { 1 };
                        
                        black_box((route.clone(), depth, response_time))
                    }
                });
            },
        );
    }

    group.finish();
}

/// Test de comparaison directe des performances par niveau
fn benchmark_nested_routes_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let server = rt.block_on(async {
        let router = create_rest_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    // Obtenir des IDs réels
    let (order_id, invoice_id) = rt.block_on(async {
        let response = server.get("/orders").await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        let orders: serde_json::Value = serde_json::from_str(&body).unwrap();
        let order_id = orders[0]["id"].as_str().unwrap().to_string();
        
        let route = format!("/orders/{}/invoices", order_id);
        let response = server.get(&route).await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        let invoices: serde_json::Value = serde_json::from_str(&body).unwrap();
        let invoice_id = invoices["data"][0]["target"]["id"].as_str().unwrap().to_string();
        
        (order_id, invoice_id)
    });

    let mut group = c.benchmark_group("nested_routes_comparison");

    // Créer les routes d'abord pour éviter les problèmes de lifetime
    let route_depth_1 = format!("/orders/{}", order_id);
    let route_depth_2 = format!("/orders/{}/invoices", order_id);
    let route_depth_3 = format!("/orders/{}/invoices/{}", order_id, invoice_id);

    // Benchmark de chaque niveau avec multiples mesures
    let test_cases = vec![
        ("depth_0", "/orders", 0),
        ("depth_1", &route_depth_1, 1),
        ("depth_2", &route_depth_2, 2),
        ("depth_3", &route_depth_3, 3),
    ];

    for (name, route, depth) in test_cases {
        group.bench_function(name, |b| {
            let server = server.clone();
            let route = route.to_string();
            
            b.to_async(&rt).iter(|| {
                let server = server.clone();
                let route = route.clone();
                async move {
                    let start = std::time::Instant::now();
                    let response = server.get(&route).await.unwrap();
                    let body = TestServer::response_body_string(response).await.unwrap();
                    let response_time = start.elapsed();
                    
                    println!("📊 Niveau {} | {}µs | Route: {}", 
                            depth, response_time.as_micros(), route);
                    
                    black_box((response_time, body.len()))
                }
            });
        });
    }

    group.finish();
}

/// Test de concurrence sur routes imbriquées
fn benchmark_nested_routes_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let server = rt.block_on(async {
        let router = create_rest_server_in_memory().await.unwrap();
        start_test_server(router).await.unwrap()
    });

    let (order_id, invoice_id) = rt.block_on(async {
        let response = server.get("/orders").await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        let orders: serde_json::Value = serde_json::from_str(&body).unwrap();
        let order_id = orders[0]["id"].as_str().unwrap().to_string();
        
        let route = format!("/orders/{}/invoices", order_id);
        let response = server.get(&route).await.unwrap();
        let body = TestServer::response_body_string(response).await.unwrap();
        let invoices: serde_json::Value = serde_json::from_str(&body).unwrap();
        let invoice_id = invoices["data"][0]["target"]["id"].as_str().unwrap().to_string();
        
        (order_id, invoice_id)
    });

    let mut group = c.benchmark_group("nested_routes_load");

    // Test de charge sur route simple vs route imbriquée
    let routes = vec![
        ("simple_orders", "/orders".to_string(), 0),
        ("nested_invoices", format!("/orders/{}/invoices", order_id), 2),
        ("deep_nested", format!("/orders/{}/invoices/{}", order_id, invoice_id), 3),
    ];

    for (name, route, depth) in routes {
        for concurrent in [10, 25, 50].iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("{}_concurrent", name), concurrent),
                &(route.clone(), *concurrent),
                |b, (route, concurrent)| {
                    let server = server.clone();
                    b.to_async(&rt).iter(|| {
                        let server = server.clone();
                        let route = route.clone();
                        async move {
                            let start = std::time::Instant::now();
                            
                            let tasks = (0..*concurrent).map(|_| {
                                let server = server.clone();
                                let route = route.clone();
                                tokio::spawn(async move {
                                    let response = server.get(&route).await.unwrap();
                                    TestServer::response_body_string(response).await.unwrap()
                                })
                            });
                            
                            let results = futures::future::join_all(tasks).await;
                            let total_time = start.elapsed();
                            
                            // Calculer le throughput
                            let rps = *concurrent as f64 / total_time.as_secs_f64();
                            
                            println!("🚀 Niveau {} | {} concurrent | {}µs total | {:.1} req/s", 
                                    depth, concurrent, total_time.as_micros(), rps);
                            
                            black_box((results, total_time))
                        }
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_nested_routes_simple,
    benchmark_nested_routes_comparison,
    benchmark_nested_routes_load
);

criterion_main!(benches);