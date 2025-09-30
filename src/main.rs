use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
// use std::env;

// mod backfill;
// mod ingest;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[get("/api/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // let args: Vec<String> = env::args().collect();
    // if args.contains(&"--backfill".to_string()) {
    //     println!("Backfill mode detected.");
    //     if let Err(e) = backfill::backfill().await {
    //         eprintln!("Backfill failed: {}", e);
    //     }
    //     return Ok(());
    // }

    // tokio::spawn(async {
    //     if let Err(e) = ingest::start_ingest().await {
    //         eprintln!("Ingest process failed: {}", e);
    //     }
    // });

    HttpServer::new(|| App::new().service(health))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}