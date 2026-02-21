use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::env;
use tracing::info;

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://papermarket.db".into());

    info!("Conectando ao banco de dados: {}", database_url);

    // ─── Criar pool de conexões ──────────────────────────────
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // ─── Rodar migrations automaticamente ───────────────────
    info!("Rodando migrations...");
    sqlx::migrate!("src/db/migrations")
        .run(&pool)
        .await?;

    info!("✅ Banco de dados pronto");

    Ok(pool)
}