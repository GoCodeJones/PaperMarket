use sqlx::SqlitePool;
use chrono::Utc;
use std::env;

use crate::errors::AppError;
use crate::blockchain::pow::{sha256_hex, mine_block};

// ─── Buscar estado atual do topo da chain ────────────────────
// Retorna (prev_hash, height, difficulty)
pub async fn get_latest_block(
    pool: &SqlitePool,
) -> Result<(String, i64, i64), AppError> {

    let row = sqlx::query!(
        "SELECT id, height, difficulty FROM blocks ORDER BY height DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(b) => Ok((b.id, b.height, b.difficulty)),
        None    => {
            // Chain vazia — retornar valores do bloco gênesis
            Ok((genesis_hash(), 0, initial_difficulty()))
        }
    }
}

// ─── Criar bloco gênesis (chamado na inicialização) ──────────
pub async fn create_genesis_block(pool: &SqlitePool) -> Result<(), AppError> {

    // Verificar se já existe
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM blocks")
        .fetch_one(pool)
        .await?;

    if count > 0 {
        return Ok(());
    }

    let difficulty  = initial_difficulty();
    let timestamp   = Utc::now().to_rfc3339();
    let prev_hash   = "0000000000000000000000000000000000000000000000000000000000000000";
    let merkle_root = sha256_hex("genesis");
    let reward_sats = initial_reward();

    tracing::info!("Minerando bloco gênesis (dificuldade {})...", difficulty);

    let (nonce, block_hash) = mine_block(
        0,
        prev_hash,
        &merkle_root,
        difficulty,
        &timestamp,
    );

    tracing::info!("Bloco gênesis minerado: {} (nonce: {})", block_hash, nonce);

    sqlx::query(
        "INSERT INTO blocks (id, height, prev_hash, merkle_root, nonce, difficulty, reward_sats, miner_address, tx_count, mined_at)
         VALUES (?, 0, ?, ?, ?, ?, ?, 'GENESIS', 0, ?)",
    )
    .bind(&block_hash)
    .bind(prev_hash)
    .bind(&merkle_root)
    .bind(nonce)
    .bind(difficulty)
    .bind(reward_sats)
    .bind(&timestamp)
    .execute(pool)
    .await?;

    Ok(())
}

// ─── Ajuste de dificuldade ───────────────────────────────────
pub async fn calculate_difficulty(pool: &SqlitePool) -> Result<i64, AppError> {
    let adjustment_interval: i64 = env::var("CHAIN_DIFFICULTY_ADJUSTMENT_INTERVAL")
        .unwrap_or_else(|_| "2016".into())
        .parse()
        .unwrap_or(2016);

    let height = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(MAX(height), 0) FROM blocks",
    )
    .fetch_one(pool)
    .await?;

    // Ajuste só acontece a cada `adjustment_interval` blocos
    if height % adjustment_interval != 0 || height == 0 {
        return Ok(
            sqlx::query_scalar::<_, i64>(
                "SELECT COALESCE(difficulty, 4) FROM blocks ORDER BY height DESC LIMIT 1",
            )
            .fetch_one(pool)
            .await
            .unwrap_or(initial_difficulty())
        );
    }

    // Tempo esperado: 10min por bloco × adjustment_interval
    let expected_time_secs = adjustment_interval * 600;

    // Tempo real dos últimos `adjustment_interval` blocos
    let oldest = sqlx::query_scalar::<_, String>(
        "SELECT mined_at FROM blocks ORDER BY height ASC LIMIT 1 OFFSET ?",
    )
    .bind(height - adjustment_interval)
    .fetch_optional(pool)
    .await?;

    let newest = sqlx::query_scalar::<_, String>(
        "SELECT mined_at FROM blocks ORDER BY height DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    let current_difficulty = sqlx::query_scalar::<_, i64>(
        "SELECT difficulty FROM blocks ORDER BY height DESC LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(initial_difficulty());

    if let (Some(old), Some(new)) = (oldest, newest) {
        let old_ts = chrono::DateTime::parse_from_rfc3339(&old)
            .map(|d| d.timestamp())
            .unwrap_or(0);
        let new_ts = chrono::DateTime::parse_from_rfc3339(&new)
            .map(|d| d.timestamp())
            .unwrap_or(0);

        let actual_time_secs = new_ts - old_ts;

        if actual_time_secs == 0 {
            return Ok(current_difficulty);
        }

        // Ajustar proporcionalmente (limitado a 4x para cima ou baixo)
        let ratio = expected_time_secs as f64 / actual_time_secs as f64;
        let ratio = ratio.clamp(0.25, 4.0);

        let new_difficulty = ((current_difficulty as f64 * ratio).round() as i64)
            .max(1)  // mínimo 1
            .min(64); // máximo 64

        tracing::info!(
            "📊 Ajuste de dificuldade: {} → {} (ratio: {:.2})",
            current_difficulty, new_difficulty, ratio
        );

        return Ok(new_difficulty);
    }

    Ok(current_difficulty)
}

// ─── Hash do bloco gênesis ───────────────────────────────────
pub fn genesis_hash() -> String {
    "0000000000000000000000000000000000000000000000000000000000000000".into()
}

// ─── Helpers de configuração ─────────────────────────────────
fn initial_difficulty() -> i64 {
    env::var("CHAIN_INITIAL_DIFFICULTY")
        .unwrap_or_else(|_| "4".into())
        .parse()
        .unwrap_or(4)
}

fn initial_reward() -> i64 {
    env::var("CHAIN_BLOCK_REWARD")
        .unwrap_or_else(|_| "625000000".into())
        .parse()
        .unwrap_or(625_000_000)
}