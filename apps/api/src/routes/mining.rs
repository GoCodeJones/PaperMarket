use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::SqlitePool;
use std::env;

use crate::errors::AppError;
use crate::models::user::Claims;
use crate::models::block::{Block, MiningJob, MiningSubmit};
use crate::models::transaction::{Transaction, Utxo};
use crate::blockchain::pow::{sha256_hex, meets_difficulty};
use crate::blockchain::mempool::get_pending_transactions;
use crate::blockchain::chain::get_latest_block;

// ─── Configuração das rotas ──────────────────────────────────
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/mining")
            .route("/job",    web::get().to(get_job))
            .route("/submit", web::post().to(submit_block)),
    );
}

// ─── GET /api/mining/job ─────────────────────────────────────
async fn get_job(
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse, AppError> {

    // Buscar último bloco
    let (prev_hash, height, difficulty) = get_latest_block(pool.as_ref()).await?;
    let next_height = height + 1;

    // Calcular recompensa com halving
    let initial_reward: i64 = env::var("CHAIN_BLOCK_REWARD")
        .unwrap_or_else(|_| "625000000".into())
        .parse()
        .unwrap_or(625_000_000);

    let halving_interval: i64 = env::var("CHAIN_HALVING_INTERVAL")
        .unwrap_or_else(|_| "210000".into())
        .parse()
        .unwrap_or(210_000);

    let halvings = next_height / halving_interval;
    let reward_sats = initial_reward >> halvings;

    // Buscar TXs pendentes da mempool
    let pending_txs = get_pending_transactions(pool.as_ref(), 100).await?;

    // Calcular merkle root das TXs
    let merkle_root = compute_merkle_root(&pending_txs);

    // Target: string de zeros de acordo com a dificuldade
    let target = "0".repeat(difficulty as usize);

    Ok(HttpResponse::Ok().json(MiningJob {
        block_height: next_height,
        prev_hash,
        merkle_root,
        difficulty,
        target,
        reward_sats,
    }))
}

// ─── POST /api/mining/submit ─────────────────────────────────
async fn submit_block(
    pool: web::Data<SqlitePool>,
    req:  HttpRequest,
    body: web::Json<MiningSubmit>,
) -> Result<HttpResponse, AppError> {

    // 1. Extrair claims do JWT
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    // 2. Buscar estado atual da chain
    let (prev_hash, height, difficulty) = get_latest_block(pool.as_ref()).await?;

    // 3. Validar altura do bloco
    if body.block_height != height + 1 {
        return Err(AppError::InvalidBlock(
            format!("Altura inválida: esperado {}, recebido {}", height + 1, body.block_height)
        ));
    }

    // 4. Validar prev_hash
    if body.prev_hash != prev_hash {
        return Err(AppError::InvalidBlock("prev_hash não confere com o topo da chain".into()));
    }

    // 5. Recompor o header e verificar o hash (PoW)
    let header_data = format!(
        "{}{}{}{}{}{}",
        body.block_height,
        body.prev_hash,
        body.merkle_root,
        body.nonce,
        difficulty,
        body.timestamp,
    );
    let block_hash = sha256_hex(&header_data);

    // 6. Verificar se o hash atende à dificuldade
    if !meets_difficulty(&block_hash, difficulty) {
        return Err(AppError::InvalidBlock(
            format!("Hash {} não atende à dificuldade {}", block_hash, difficulty)
        ));
    }

    // 7. Calcular recompensa
    let initial_reward: i64 = env::var("CHAIN_BLOCK_REWARD")
        .unwrap_or_else(|_| "625000000".into())
        .parse()
        .unwrap_or(625_000_000);

    let halving_interval: i64 = env::var("CHAIN_HALVING_INTERVAL")
        .unwrap_or_else(|_| "210000".into())
        .parse()
        .unwrap_or(210_000);

    let halvings = body.block_height / halving_interval;
    let reward_sats = initial_reward >> halvings;

    // 8. Buscar TXs pendentes para incluir no bloco
    let pending_txs = get_pending_transactions(pool.as_ref(), 100).await?;
    let tx_count = pending_txs.len() as i64;

    // 9. Salvar bloco no banco
    let block = Block::new(
        block_hash.clone(),
        body.block_height,
        body.prev_hash.clone(),
        body.merkle_root.clone(),
        body.nonce,
        difficulty,
        reward_sats,
        claims.address.clone(),
        tx_count,
    );

    sqlx::query(
        "INSERT INTO blocks (id, height, prev_hash, merkle_root, nonce, difficulty, reward_sats, miner_address, tx_count, mined_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&block.id)
    .bind(block.height)
    .bind(&block.prev_hash)
    .bind(&block.merkle_root)
    .bind(block.nonce)
    .bind(block.difficulty)
    .bind(block.reward_sats)
    .bind(&block.miner_address)
    .bind(block.tx_count)
    .bind(&block.mined_at)
    .execute(pool.as_ref())
    .await?;

    // 10. Confirmar TXs pendentes
    for tx in &pending_txs {
        sqlx::query(
            "UPDATE transactions SET status = 'confirmed', block_id = ? WHERE id = ?",
        )
        .bind(&block_hash)
        .bind(&tx.id)
        .execute(pool.as_ref())
        .await?;

        // Criar UTXO para o destinatário
        let utxo = Utxo::new(tx.id.clone(), tx.receiver.clone(), tx.amount_sats);
        sqlx::query(
            "INSERT INTO utxos (id, tx_id, owner, amount_sats, spent, spent_tx_id, created_at)
             VALUES (?, ?, ?, ?, 0, NULL, ?)",
        )
        .bind(&utxo.id)
        .bind(&utxo.tx_id)
        .bind(&utxo.owner)
        .bind(utxo.amount_sats)
        .bind(&utxo.created_at)
        .execute(pool.as_ref())
        .await?;
    }

    // 11. Criar UTXO de recompensa para o minerador
    let reward_utxo = Utxo::new(block_hash.clone(), claims.address.clone(), reward_sats);
    sqlx::query(
        "INSERT INTO utxos (id, tx_id, owner, amount_sats, spent, spent_tx_id, created_at)
         VALUES (?, ?, ?, ?, 0, NULL, ?)",
    )
    .bind(&reward_utxo.id)
    .bind(&reward_utxo.tx_id)
    .bind(&reward_utxo.owner)
    .bind(reward_utxo.amount_sats)
    .bind(&reward_utxo.created_at)
    .execute(pool.as_ref())
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message":      "Bloco aceito!",
        "block_hash":   block_hash,
        "height":       body.block_height,
        "reward_sats":  reward_sats,
        "txs_included": tx_count,
    })))
}

// ─── Calcular Merkle Root simples ────────────────────────────
fn compute_merkle_root(txs: &[Transaction]) -> String {
    if txs.is_empty() {
        return sha256_hex("empty");
    }

    let mut hashes: Vec<String> = txs.iter().map(|tx| tx.id.clone()).collect();

    while hashes.len() > 1 {
        if hashes.len() % 2 != 0 {
            hashes.push(hashes.last().unwrap().clone());
        }
        hashes = hashes
            .chunks(2)
            .map(|pair| sha256_hex(&format!("{}{}", pair[0], pair[1])))
            .collect();
    }

    hashes.into_iter().next().unwrap_or_else(|| sha256_hex("empty"))
}