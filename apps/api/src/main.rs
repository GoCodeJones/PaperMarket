use actix_web::{web, App, HttpServer, middleware};
use dotenvy::dotenv;
use std::env;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
mod models;
mod blockchain;
mod crypto;
mod db;
mod ws;
mod errors;

use db::connection::init_db;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ─── Carregar .env ───────────────────────────────────────
    dotenv().ok();

    // ─── Inicializar logs ────────────────────────────────────
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ─── Configurações do servidor ───────────────────────────
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("{}:{}", host, port);

    // ─── Conectar ao banco de dados ──────────────────────────
    let pool = init_db().await.expect("Falha ao conectar ao banco de dados");
    let pool = web::Data::new(pool);

    info!("🚀 PaperMarket API rodando em http://{}", addr);

    // ─── Iniciar servidor ────────────────────────────────────
    HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allowed_origin(
                &env::var("CORS_ALLOWED_ORIGIN")
                    .unwrap_or_else(|_| "http://localhost:3000".into()),
            )
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(pool.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            // ─── Rotas ──────────────────────────────────────
            .service(
                web::scope("/api")
                    .configure(routes::auth::config)
                    .configure(routes::wallet::config)
                    .configure(routes::marketplace::config)
                    .configure(routes::chain::config)
                    .configure(routes::mining::config)
                    .configure(routes::contracts::config)
            )
            // ─── WebSocket ───────────────────────────────────
            .route("/ws", web::get().to(ws::handler::ws_handler))
    })
    .bind(&addr)?
    .run()
    .await
}