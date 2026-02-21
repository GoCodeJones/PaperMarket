use actix_web::{web, HttpResponse};
use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::block::{Block, BlockResponse, ChainInfo};
use crate::models::transaction::Transaction;

// ─── Configuração das rotas ──────────────────────────────────
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/chain")
            .route("/info",              web::get().to(get_chain_info))
            .route("/blocks",            web::get().to(list_blocks))
            .route("/blocks/{height}",   web::get().to(get_block))
            .route("/tx/{hash}",         web::get().to(get_transaction)),
    );
}

// ─── GET /api/chain/info ─────────────────────────────────────
async fn get_chain_info(
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse, AppError> {

    // Altura atual
    let height = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(MAX(height), 0) FROM blocks",
    )
    .fetch_one(pool.as_ref())
    .await?;

    // Hash do melhor bloco
    let best_hash = sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(id, '0000000000000000') FROM blocks ORDER BY height DESC LIMIT 1",
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or_else(|_| "0000000000000000".into());

    // Dificuldade atual
    let difficulty = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(difficulty, 4) FROM blocks ORDER BY height DESC LIMIT 1",
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(4);

    // Total emitido em satoshis
    let total_supply = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(reward_sats), 0) FROM blocks",
    )
    .fetch_one(pool.as_ref())
    .await?;

    // TXs pendentes na mempool
    let mempool_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM transactions WHERE status = 'pending'",
    )
    .fetch_one(pool.as_ref())
    .await?;

    // Recompensa atual (considera halving)
    let initial_reward: i64 = std::env::var("CHAIN_BLOCK_REWARD")
        .unwrap_or_else(|_| "625000000".into())
        .parse()
        .unwrap_or(625_000_000);

    let halving_interval: i64 = std::env::var("CHAIN_HALVING_INTERVAL")
        .unwrap_or_else(|_| "210000".into())
        .parse()
        .unwrap_or(210_000);

    let halvings = height / halving_interval;
    let block_reward = initial_reward >> halvings;

    Ok(HttpResponse::Ok().json(ChainInfo {
        height,
        best_hash,
        difficulty,
        total_supply,
        mempool_count,
        block_reward,
    }))
}

// ─── GET /api/chain/blocks ───────────────────────────────────
async fn list_blocks(
    pool:  web::Data<SqlitePool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, AppError> {
    let page  = query.get("page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(1);
    let limit = query.get("limit").and_then(|l| l.parse::<i64>().ok()).unwrap_or(20);
    let offset = (page - 1) * limit;

    let blocks = sqlx::query_as::<_, Block>(
        "SELECT * FROM blocks ORDER BY height DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.as_ref())
    .await?;

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM blocks")
        .fetch_one(pool.as_ref())
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "blocks": blocks,
        "total":  total,
        "page":   page,
    })))
}

// ─── GET /api/chain/blocks/:height ──────────────────────────
async fn get_block(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let height = path.into_inner();

    let block = sqlx::query_as::<_, Block>(
        "SELECT * FROM blocks WHERE height = ?",
    )
    .bind(height)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Bloco #{} não encontrado", height)))?;

    // Buscar TXs do bloco
    let tx_ids = sqlx::query_scalar::<_, String>(
        "SELECT id FROM transactions WHERE block_id = ?",
    )
    .bind(&block.id)
    .fetch_all(pool.as_ref())
    .await?;

    Ok(HttpResponse::Ok().json(BlockResponse {
        block,
        transactions: tx_ids,
    }))
}

// ─── GET /api/chain/tx/:hash ─────────────────────────────────
async fn get_transaction(
    pool: web::Data<SqlitePool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let hash = path.into_inner();

    let tx = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions WHERE id = ?",
    )
    .bind(&hash)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Transação {} não encontrada", hash)))?;

    Ok(HttpResponse::Ok().json(tx))
}