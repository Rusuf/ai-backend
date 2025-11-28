use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Context;
use serde::Serialize;
use sqlx::{MySqlPool, PgPool};
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

mod ml;
mod recommendations;
mod stock_optimization;
mod trend_discovery;
mod market_intelligence;
mod sync;
mod agent;


// Define a struct to hold our application state
struct AppState {
    pool: PgPool,
    recommendation_cache: RwLock<HashMap<i32, Vec<recommendations::Recommendation>>>,
}

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

#[get("/api/recommendations/{product_id}")]
async fn get_recommendations(
    state: web::Data<AppState>,
    product_id: web::Path<i32>,
) -> impl Responder {
    let product_id = product_id.into_inner();
    let cache = state.recommendation_cache.read().await;
    let recommendations = recommendations::get_recommendations_from_cache(
        &cache,
        product_id,
    );
    HttpResponse::Ok().json(recommendations)
}

#[post("/api/retrain")]
async fn retrain_model(state: web::Data<AppState>) -> impl Responder {
    info!("Manual retraining triggered via API...");
    match recommendations::train_and_cache_recommendations(&state.pool).await {
        Ok(new_cache) => {
            let count = new_cache.len();
            let mut cache = state.recommendation_cache.write().await;
            *cache = new_cache;
            info!("Retraining complete. Cache updated with {} items.", count);
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": format!("Model retrained. {} items cached.", count)
            }))
        }
        Err(e) => {
            error!("Retraining failed: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": e.to_string()
            }))
        }
    }
}

#[get("/api/stock_optimization")]
async fn get_stock_optimization(state: web::Data<AppState>) -> impl Responder {
    match stock_optimization::get_stock_optimization(&state.pool).await {
        Ok(optimization) => HttpResponse::Ok().json(optimization),
        Err(e) => {
            error!("Stock optimization error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/api/trending_recipes")]
async fn get_trending_recipes(state: web::Data<AppState>) -> impl Responder {
    match trend_discovery::get_trending_recipes(&state.pool).await {
        Ok(recipes) => HttpResponse::Ok().json(recipes),
        Err(e) => {
            error!("Trending recipes error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}


#[get("/api/market_intelligence")]
async fn get_market_intelligence(state: web::Data<AppState>) -> impl Responder {
    match market_intelligence::get_market_intelligence(&state.pool).await {
        Ok(trends) => HttpResponse::Ok().json(trends),
        Err(e) => {
            error!("Market intelligence error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Ruby AI Backend...");

    // --- Connect to Databases ---
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pg_pool = PgPool::connect(&database_url)
        .await
        .context("Failed to create PG pool")?;

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pg_pool)
        .await
        .context("Failed to run migrations")?;
    info!("Migrations applied successfully.");

    // --- Connect to MySQL (Optional for Demo) ---
    let mysql_dsn = env::var("MYSQL_DSN").context("MYSQL_DSN must be set")?;
    let mysql_pool = match MySqlPool::connect(&mysql_dsn).await {
        Ok(pool) => {
            info!("Connected to MySQL successfully.");
            Some(pool)
        }
        Err(e) => {
            error!("Failed to connect to MySQL: {}.", e);
            warn!("⚠️ Reverting to local DB mode. Sync will be disabled, but API endpoints will function using local data.");
            None
        }

    };

    // --- Train the recommendation model at startup ---
    info!("Training recommendation model on server startup...");
    let recommendation_cache = match recommendations::train_and_cache_recommendations(&pg_pool).await {
        Ok(cache) => {
            info!("Recommendation model is ready with {} items cached.", cache.len());
            cache
        }
        Err(e) => {
            error!("Failed to train recommendation model: {}. Using empty cache.", e);
            HashMap::new()
        }
    };

    // --- Spawn the Market Intelligence Agent ---
    let agent_pool = pg_pool.clone();
    tokio::spawn(async move {
        agent::scraper::run_market_agent(agent_pool).await;
    });

    // --- Spawn the periodic database sync task ---
    if let Some(mysql_pool) = mysql_pool {
        let background_pg_pool = pg_pool.clone();
        let background_mysql_pool = mysql_pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1800)); // 30 minutes
            loop {
                interval.tick().await;
                info!("Running periodic database synchronization...");
                if let Err(e) = sync::process_changes(&background_mysql_pool, &background_pg_pool).await {
                    error!("Error during periodic sync: {:?}", e);
                }
                info!("Synchronization check complete.");
            }
        });
    } else {
        warn!("MySQL pool not available. Periodic sync is disabled.");
    }

    // --- Create the application state ---
    let app_state = web::Data::new(AppState {
        pool: pg_pool.clone(),
        recommendation_cache: RwLock::new(recommendation_cache),
    });


    info!("Starting Actix web server at http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(health)
            .service(get_recommendations)
            .service(retrain_model)
            .service(get_stock_optimization)
            .service(get_trending_recipes)
            .service(get_market_intelligence)
    })

    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}

