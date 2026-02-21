use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::transaction::Transaction;

// ─── Buscar TXs pendentes (ordenadas por taxa) ───────────────
pub async fn get_pending_transactions(
    pool:  &SqlitePool,
    limit: i64,
) -> Result<Vec<Transaction>, AppError> {
    let txs = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions
         WHERE status = 'pending'
         ORDER BY fee_sats DESC, created_at ASC
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(txs)
}

// ─── Quantidade de TXs na mempool ────────────────────────────
pub async fn mempool_count(pool: &SqlitePool) -> Result<i64, AppError> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM transactions WHERE status = 'pending'",
    )
    .fetch_one(pool)
    .await?;

    Ok(count)
}

// ─── Taxa média da mempool (sat/vB) ──────────────────────────
pub async fn average_fee(pool: &SqlitePool) -> Result<i64, AppError> {
    let avg = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT CAST(AVG(fee_sats) AS INTEGER)
         FROM transactions
         WHERE status = 'pending'",
    )
    .fetch_one(pool)
    .await?;

    Ok(avg.unwrap_or(0))
}

// ─── Taxa mínima recomendada ─────────────────────────────────
pub async fn recommended_fee(pool: &SqlitePool) -> Result<i64, AppError> {
    let count = mempool_count(pool).await?;

    // Fee dinâmica baseada no volume da mempool
    let fee = match count {
        0..=10   =>  5,   // mempool vazia — fee baixa
        11..=50  =>  10,  // moderada
        51..=100 =>  20,  // congestionada
        _        =>  50,  // muito congestionada
    };

    Ok(fee)
}

// ─── Remover TXs antigas da mempool (limpeza) ────────────────
pub async fn evict_stale_transactions(
    pool:            &SqlitePool,
    max_age_minutes: i64,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "UPDATE transactions
         SET status = 'rejected'
         WHERE status = 'pending'
         AND created_at < datetime('now', ? || ' minutes')",
    )
    .bind(format!("-{}", max_age_minutes))
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// ─── Verificar se TX já existe na mempool ────────────────────
pub async fn tx_exists(pool: &SqlitePool, tx_id: &str) -> Result<bool, AppError> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM transactions WHERE id = ?",
    )
    .bind(tx_id)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}