use sqlx::SqlitePool;
use chrono::Utc;

use crate::errors::AppError;
use crate::models::contract::{Contract, ContractEvent, ContractState};
use crate::blockchain::chain::get_latest_block;

// ─── Verificar contratos expirados e reembolsar ──────────────
pub async fn process_expired_contracts(pool: &SqlitePool) -> Result<u64, AppError> {
    let (_, current_height, _) = get_latest_block(pool).await?;
    let now = Utc::now().to_rfc3339();
    let mut refunded = 0u64;

    // Buscar contratos PENDING ou LOCKED que expiraram
    let expired = sqlx::query_as::<_, Contract>(
        "SELECT * FROM contracts
         WHERE state IN ('PENDING', 'LOCKED')
         AND expires_at_block <= ?",
    )
    .bind(current_height)
    .fetch_all(pool)
    .await?;

    for contract in expired {
        // Atualizar estado para REFUNDED
        sqlx::query(
            "UPDATE contracts SET state = 'REFUNDED', updated_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(&contract.id)
        .execute(pool)
        .await?;

        // Registrar evento REFUNDED
        let event = ContractEvent::new(
            contract.id.clone(),
            "REFUNDED".into(),
            Some(format!(
                "Contrato expirado no bloco {}. Fundos devolvidos ao comprador.",
                current_height
            )),
        );

        sqlx::query(
            "INSERT INTO contract_events (id, contract_id, event_type, description, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(&event.contract_id)
        .bind(&event.event_type)
        .bind(&event.description)
        .bind(&event.created_at)
        .execute(pool)
        .await?;

        refunded += 1;
        tracing::info!("Contrato {} reembolsado (expirado no bloco {})", contract.id, current_height);
    }

    Ok(refunded)
}

// ─── Travar fundos do contrato (PENDING → LOCKED) ────────────
pub async fn lock_contract(
    pool:      &SqlitePool,
    id:        &str,
    lock_tx_id: &str,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();

    let affected = sqlx::query(
        "UPDATE contracts SET state = 'LOCKED', lock_tx_id = ?, updated_at = ?
         WHERE id = ? AND state = 'PENDING'",
    )
    .bind(lock_tx_id)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::InvalidContractState(
            "Contrato não encontrado ou não está em estado PENDING".into(),
        ));
    }

    // Registrar evento LOCKED
    let event = ContractEvent::new(
        id.to_string(),
        "LOCKED".into(),
        Some(format!("Fundos travados na TX {}", lock_tx_id)),
    );

    sqlx::query(
        "INSERT INTO contract_events (id, contract_id, event_type, description, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&event.id)
    .bind(&event.contract_id)
    .bind(&event.event_type)
    .bind(&event.description)
    .bind(&event.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

// ─── Liberar fundos do contrato (→ RELEASED) ─────────────────
pub async fn release_contract(
    pool:           &SqlitePool,
    id:             &str,
    release_tx_id:  &str,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();

    let affected = sqlx::query(
        "UPDATE contracts SET state = 'RELEASED', release_tx_id = ?, updated_at = ?
         WHERE id = ? AND state IN ('LOCKED', 'DISPUTED')",
    )
    .bind(release_tx_id)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::InvalidContractState(
            "Contrato não encontrado ou não pode ser liberado".into(),
        ));
    }

    // Registrar evento RELEASED
    let event = ContractEvent::new(
        id.to_string(),
        "RELEASED".into(),
        Some(format!("Fundos liberados ao vendedor na TX {}", release_tx_id)),
    );

    sqlx::query(
        "INSERT INTO contract_events (id, contract_id, event_type, description, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&event.id)
    .bind(&event.contract_id)
    .bind(&event.event_type)
    .bind(&event.description)
    .bind(&event.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

// ─── Estatísticas de contratos ───────────────────────────────
pub async fn contract_stats(pool: &SqlitePool) -> Result<serde_json::Value, AppError> {
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM contracts",
    )
    .fetch_one(pool)
    .await?;

    let by_state = sqlx::query!(
        "SELECT state, COUNT(*) as count FROM contracts GROUP BY state"
    )
    .fetch_all(pool)
    .await?;

    let volume = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(amount_sats), 0) FROM contracts WHERE state = 'RELEASED'",
    )
    .fetch_one(pool)
    .await?;

    let fees = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(fee_sats), 0) FROM contracts WHERE state = 'RELEASED'",
    )
    .fetch_one(pool)
    .await?;

    let states: serde_json::Value = by_state
        .iter()
        .map(|r| (r.state.clone(), r.count))
        .collect::<std::collections::HashMap<_, _>>()
        .into();

    Ok(serde_json::json!({
        "total":        total,
        "by_state":     states,
        "volume_sats":  volume,
        "fees_sats":    fees,
    }))
}