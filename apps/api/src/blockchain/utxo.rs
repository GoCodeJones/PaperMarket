use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::transaction::Utxo;

// ─── Buscar saldo confirmado de um endereço ──────────────────
pub async fn get_balance(
    pool:    &SqlitePool,
    address: &str,
) -> Result<i64, AppError> {
    let balance = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(amount_sats), 0)
         FROM utxos
         WHERE owner = ? AND spent = 0",
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    Ok(balance)
}

// ─── Buscar UTXOs não gastos de um endereço ──────────────────
pub async fn get_utxos(
    pool:    &SqlitePool,
    address: &str,
) -> Result<Vec<Utxo>, AppError> {
    let utxos = sqlx::query_as::<_, Utxo>(
        "SELECT * FROM utxos
         WHERE owner = ? AND spent = 0
         ORDER BY amount_sats DESC",
    )
    .bind(address)
    .fetch_all(pool)
    .await?;

    Ok(utxos)
}

// ─── Selecionar UTXOs suficientes para cobrir um valor ───────
// Algoritmo simples: maior UTXO primeiro (greedy)
pub async fn select_utxos(
    pool:        &SqlitePool,
    address:     &str,
    target_sats: i64,
) -> Result<Vec<Utxo>, AppError> {
    let utxos = get_utxos(pool, address).await?;

    let mut selected  = Vec::new();
    let mut total     = 0i64;

    for utxo in utxos {
        if total >= target_sats {
            break;
        }
        total += utxo.amount_sats;
        selected.push(utxo);
    }

    if total < target_sats {
        return Err(AppError::InsufficientBalance);
    }

    Ok(selected)
}

// ─── Marcar UTXOs como gastos ────────────────────────────────
pub async fn spend_utxos(
    pool:     &SqlitePool,
    utxo_ids: &[String],
    tx_id:    &str,
) -> Result<(), AppError> {
    for utxo_id in utxo_ids {
        sqlx::query(
            "UPDATE utxos SET spent = 1, spent_tx_id = ? WHERE id = ?",
        )
        .bind(tx_id)
        .bind(utxo_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

// ─── Criar UTXO de troco ─────────────────────────────────────
pub async fn create_change_utxo(
    pool:        &SqlitePool,
    tx_id:       &str,
    owner:       &str,
    change_sats: i64,
) -> Result<(), AppError> {
    if change_sats <= 0 {
        return Ok(());
    }

    let utxo = crate::models::transaction::Utxo::new(
        tx_id.to_string(),
        owner.to_string(),
        change_sats,
    );

    sqlx::query(
        "INSERT INTO utxos (id, tx_id, owner, amount_sats, spent, spent_tx_id, created_at)
         VALUES (?, ?, ?, ?, 0, NULL, ?)",
    )
    .bind(&utxo.id)
    .bind(&utxo.tx_id)
    .bind(&utxo.owner)
    .bind(utxo.amount_sats)
    .bind(&utxo.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

// ─── Total emitido na chain ───────────────────────────────────
pub async fn total_supply(pool: &SqlitePool) -> Result<i64, AppError> {
    let supply = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(amount_sats), 0) FROM utxos",
    )
    .fetch_one(pool)
    .await?;

    Ok(supply)
}