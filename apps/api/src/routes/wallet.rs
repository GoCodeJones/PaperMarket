use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::user::Claims;
use crate::models::transaction::{
    BalanceResponse, SendRequest, SendResponse, Transaction, TxHistoryResponse, Utxo,
};
use crate::crypto::signing::verify_signature;
use crate::blockchain::utxo::get_balance;

// ─── Configuração das rotas ──────────────────────────────────
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/wallet")
            .route("/{address}",      web::get().to(get_wallet))
            .route("/{address}/txs",  web::get().to(get_transactions))
            .route("/send",           web::post().to(send)),
    );
}

// ─── GET /api/wallet/:address ────────────────────────────────
async fn get_wallet(
    pool:    web::Data<SqlitePool>,
    path:    web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let address = path.into_inner();

    // Verificar se carteira existe
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM wallets WHERE address = ?",
    )
    .bind(&address)
    .fetch_one(pool.as_ref())
    .await?;

    if exists == 0 {
        return Err(AppError::NotFound("Carteira não encontrada".into()));
    }

    // Saldo confirmado (UTXOs não gastos)
    let balance_sats = get_balance(pool.as_ref(), &address).await?;

    // Saldo pendente (TXs na mempool)
    let pending_sats = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(amount_sats), 0)
         FROM transactions
         WHERE receiver = ? AND status = 'pending'",
    )
    .bind(&address)
    .fetch_one(pool.as_ref())
    .await?;

    // Quantidade de UTXOs não gastos
    let utxo_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM utxos WHERE owner = ? AND spent = 0",
    )
    .bind(&address)
    .fetch_one(pool.as_ref())
    .await?;

    Ok(HttpResponse::Ok().json(BalanceResponse {
        address,
        balance_sats,
        pending_sats,
        utxo_count,
    }))
}

// ─── GET /api/wallet/:address/txs ───────────────────────────
async fn get_transactions(
    pool:    web::Data<SqlitePool>,
    path:    web::Path<String>,
    query:   web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, AppError> {
    let address = path.into_inner();
    let page  = query.get("page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(1);
    let limit = query.get("limit").and_then(|l| l.parse::<i64>().ok()).unwrap_or(20);
    let offset = (page - 1) * limit;

    // Total de TXs
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM transactions
         WHERE sender = ? OR receiver = ?",
    )
    .bind(&address)
    .bind(&address)
    .fetch_one(pool.as_ref())
    .await?;

    // Buscar TXs paginadas
    let transactions = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions
         WHERE sender = ? OR receiver = ?
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?",
    )
    .bind(&address)
    .bind(&address)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.as_ref())
    .await?;

    Ok(HttpResponse::Ok().json(TxHistoryResponse {
        transactions,
        total,
        page,
    }))
}

// ─── POST /api/wallet/send ───────────────────────────────────
async fn send(
    pool:    web::Data<SqlitePool>,
    req:     HttpRequest,
    body:    web::Json<SendRequest>,
) -> Result<HttpResponse, AppError> {

    // 1. Extrair claims do JWT
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    // 2. Validações básicas
    if body.amount_sats <= 0 {
        return Err(AppError::Validation("Valor deve ser maior que zero".into()));
    }
    if body.fee_sats < 0 {
        return Err(AppError::Validation("Taxa não pode ser negativa".into()));
    }
    if claims.address == body.receiver {
        return Err(AppError::Validation("Não é possível enviar para si mesmo".into()));
    }

    // 3. Verificar saldo
    let balance = get_balance(pool.as_ref(), &claims.address).await?;
    let total_needed = body.amount_sats + body.fee_sats;

    if balance < total_needed {
        return Err(AppError::InsufficientBalance);
    }

    // 4. Verificar assinatura da TX
    let sender_pubkey = sqlx::query_scalar::<_, String>(
        "SELECT pubkey FROM wallets WHERE address = ?",
    )
    .bind(&claims.address)
    .fetch_one(pool.as_ref())
    .await?;

    let message = format!(
        "{}{}{}{}",
        claims.address, body.receiver, body.amount_sats, body.fee_sats
    );

    verify_signature(&message, &body.signature, &sender_pubkey)
        .map_err(|_| AppError::InvalidSignature)?;

    // 5. Gerar hash da TX
    let tx_data = format!(
        "{}{}{}{}{}",
        claims.address, body.receiver, body.amount_sats, body.fee_sats,
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let tx_id = crate::blockchain::pow::sha256_hex(&tx_data);

    // 6. Salvar TX na mempool
    let tx = Transaction::new(
        tx_id.clone(),
        claims.address.clone(),
        body.receiver.clone(),
        body.amount_sats,
        body.fee_sats,
        body.signature.clone(),
    );

    sqlx::query(
        "INSERT INTO transactions (id, block_id, sender, receiver, amount_sats, fee_sats, signature, status, created_at)
         VALUES (?, NULL, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&tx.id)
    .bind(&tx.sender)
    .bind(&tx.receiver)
    .bind(tx.amount_sats)
    .bind(tx.fee_sats)
    .bind(&tx.signature)
    .bind(&tx.status)
    .bind(&tx.created_at)
    .execute(pool.as_ref())
    .await?;

    // 7. Marcar UTXOs como gastos
    sqlx::query(
        "UPDATE utxos SET spent = 1, spent_tx_id = ?
         WHERE owner = ? AND spent = 0
         LIMIT 1",
    )
    .bind(&tx_id)
    .bind(&claims.address)
    .execute(pool.as_ref())
    .await?;

    Ok(HttpResponse::Created().json(SendResponse {
        tx_id,
        status: "pending".into(),
        amount_sats: body.amount_sats,
        fee_sats: body.fee_sats,
    }))
}