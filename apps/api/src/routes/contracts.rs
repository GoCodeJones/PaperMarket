use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sqlx::SqlitePool;
use std::env;

use crate::errors::AppError;
use crate::models::user::Claims;
use crate::models::contract::{
    Contract, ContractEvent, ContractResponse, ContractSignature,
    ContractState, CreateEscrowRequest, DisputeRequest, SignContractRequest,
};
use crate::blockchain::pow::sha256_hex;
use crate::blockchain::chain::get_latest_block;
use crate::crypto::signing::verify_signature;

// ─── Configuração das rotas ──────────────────────────────────
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/contracts")
            .route("",              web::post().to(create_escrow))
            .route("/{id}",         web::get().to(get_contract))
            .route("/{id}/sign",    web::post().to(sign_contract))
            .route("/{id}/dispute", web::post().to(dispute_contract)),
    );
}

// ─── POST /api/contracts ─────────────────────────────────────
async fn create_escrow(
    pool: web::Data<SqlitePool>,
    req:  HttpRequest,
    body: web::Json<CreateEscrowRequest>,
) -> Result<HttpResponse, AppError> {

    // 1. Extrair claims do JWT (comprador)
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    // 2. Buscar pubkey do comprador
    let buyer_pubkey = sqlx::query_scalar::<_, String>(
        "SELECT pubkey FROM wallets WHERE address = ?",
    )
    .bind(&claims.address)
    .fetch_one(pool.as_ref())
    .await?;

    // 3. Verificar se produto existe
    let product = sqlx::query!(
        "SELECT id, title, description, price_sats, seller_id FROM products WHERE id = ? AND status = 'active'",
        body.product_id
    )
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Produto não encontrado".into()))?;

    // 4. Validar que comprador não é o vendedor
    let seller_address = sqlx::query_scalar::<_, String>(
        "SELECT address FROM wallets WHERE user_id = ?",
    )
    .bind(&product.seller_id)
    .fetch_one(pool.as_ref())
    .await?;

    if seller_address == claims.address {
        return Err(AppError::Validation("Você não pode comprar seu próprio produto".into()));
    }

    // 5. Calcular taxa do escrow
    let escrow_fee_percent: f64 = env::var("ESCROW_FEE_PERCENT")
        .unwrap_or_else(|_| "0.5".into())
        .parse()
        .unwrap_or(0.5);

    let fee_sats = (body.amount_sats as f64 * escrow_fee_percent / 100.0) as i64;

    // 6. Buscar pubkey do vendedor
    let seller_pubkey = sqlx::query_scalar::<_, String>(
        "SELECT pubkey FROM wallets WHERE user_id = ?",
    )
    .bind(&product.seller_id)
    .fetch_one(pool.as_ref())
    .await?;

    // 7. Pubkey do árbitro (fixo — PaperMarket)
    let arbiter_pubkey = env::var("ARBITER_PUBKEY")
        .unwrap_or_else(|_| "PAPERMARKET_ARB_MASTER_01".into());

    // 8. Hash do item (SHA-256 do título + descrição)
    let item_data = format!("{}{}", product.title, product.description);
    let item_hash = sha256_hex(&item_data);

    // 9. Altura atual e expiração (100 blocos)
    let (_, current_height, _) = get_latest_block(pool.as_ref()).await?;
    let created_at_block = current_height;
    let expires_at_block = current_height + 100;

    // 10. Criar contrato
    let contract = Contract::new(
        body.product_id.clone(),
        buyer_pubkey,
        seller_pubkey,
        arbiter_pubkey,
        body.amount_sats,
        fee_sats,
        item_hash,
        created_at_block,
        expires_at_block,
    );

    sqlx::query(
        "INSERT INTO contracts (id, version, product_id, buyer_pubkey, seller_pubkey, arbiter_pubkey,
         amount_sats, fee_sats, item_hash, state, created_at_block, expires_at_block,
         lock_tx_id, release_tx_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, ?, ?)",
    )
    .bind(&contract.id)
    .bind(&contract.version)
    .bind(&contract.product_id)
    .bind(&contract.buyer_pubkey)
    .bind(&contract.seller_pubkey)
    .bind(&contract.arbiter_pubkey)
    .bind(contract.amount_sats)
    .bind(contract.fee_sats)
    .bind(&contract.item_hash)
    .bind(&contract.state)
    .bind(contract.created_at_block)
    .bind(contract.expires_at_block)
    .bind(&contract.created_at)
    .bind(&contract.updated_at)
    .execute(pool.as_ref())
    .await?;

    // 11. Registrar evento CREATED
    let event = ContractEvent::new(
        contract.id.clone(),
        "CREATED".into(),
        Some(format!("Contrato criado pelo comprador. Expira no bloco {}.", expires_at_block)),
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
    .execute(pool.as_ref())
    .await?;

    Ok(HttpResponse::Created().json(&contract))
}

// ─── GET /api/contracts/:id ──────────────────────────────────
async fn get_contract(
    pool: web::Data<SqlitePool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let contract = sqlx::query_as::<_, Contract>(
        "SELECT * FROM contracts WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Contrato não encontrado".into()))?;

    let signatures = sqlx::query_as::<_, ContractSignature>(
        "SELECT * FROM contract_signatures WHERE contract_id = ?",
    )
    .bind(&id)
    .fetch_all(pool.as_ref())
    .await?;

    let events = sqlx::query_as::<_, ContractEvent>(
        "SELECT * FROM contract_events WHERE contract_id = ORDER BY created_at ASC",
    )
    .bind(&id)
    .fetch_all(pool.as_ref())
    .await?;

    let signed_by: Vec<String> = signatures.iter().map(|s| s.role.clone()).collect();
    let can_release = signed_by.len() >= 2;

    Ok(HttpResponse::Ok().json(ContractResponse {
        contract,
        signatures,
        events,
        signed_by,
        can_release,
    }))
}

// ─── POST /api/contracts/:id/sign ────────────────────────────
async fn sign_contract(
    pool: web::Data<SqlitePool>,
    req:  HttpRequest,
    path: web::Path<String>,
    body: web::Json<SignContractRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let id = path.into_inner();

    // Buscar contrato
    let contract = sqlx::query_as::<_, Contract>(
        "SELECT * FROM contracts WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Contrato não encontrado".into()))?;

    // Verificar estado
    if contract.state == ContractState::Released.as_str()
        || contract.state == ContractState::Refunded.as_str()
    {
        return Err(AppError::InvalidContractState(
            "Contrato já foi finalizado".into(),
        ));
    }

    // Buscar pubkey do signatário
    let signer_pubkey = sqlx::query_scalar::<_, String>(
        "SELECT pubkey FROM wallets WHERE address = ?",
    )
    .bind(&claims.address)
    .fetch_one(pool.as_ref())
    .await?;

    // Determinar papel do signatário
    let role = if signer_pubkey == contract.buyer_pubkey {
        "buyer"
    } else if signer_pubkey == contract.seller_pubkey {
        "seller"
    } else if signer_pubkey == contract.arbiter_pubkey {
        "arbiter"
    } else {
        return Err(AppError::Unauthorized);
    };

    // Verificar se já assinou
    let already_signed = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM contract_signatures WHERE contract_id = ? AND role = ?",
    )
    .bind(&id)
    .bind(role)
    .fetch_one(pool.as_ref())
    .await?;

    if already_signed > 0 {
        return Err(AppError::AlreadyExists("Você já assinou este contrato".into()));
    }

    // Verificar assinatura
    verify_signature(&id, &body.signature, &signer_pubkey)
        .map_err(|_| AppError::InvalidSignature)?;

    // Salvar assinatura
    let sig = ContractSignature::new(
        id.clone(),
        signer_pubkey,
        body.signature.clone(),
        role.into(),
    );

    sqlx::query(
        "INSERT INTO contract_signatures (id, contract_id, signer_pubkey, signature, role, signed_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&sig.id)
    .bind(&sig.contract_id)
    .bind(&sig.signer_pubkey)
    .bind(&sig.signature)
    .bind(&sig.role)
    .bind(&sig.signed_at)
    .execute(pool.as_ref())
    .await?;

    // Registrar evento SIGNED
    let event = ContractEvent::new(
        id.clone(),
        "SIGNED".into(),
        Some(format!("Assinado pelo {}", role)),
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
    .execute(pool.as_ref())
    .await?;

    // Verificar se atingiu 2/3 assinaturas → liberar fundos
    let sig_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM contract_signatures WHERE contract_id = ?",
    )
    .bind(&id)
    .fetch_one(pool.as_ref())
    .await?;

    if sig_count >= 2 {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE contracts SET state = 'RELEASED', updated_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(&id)
        .execute(pool.as_ref())
        .await?;

        let release_event = ContractEvent::new(
            id.clone(),
            "RELEASED".into(),
            Some("2/3 assinaturas atingidas. Fundos liberados ao vendedor.".into()),
        );

        sqlx::query(
            "INSERT INTO contract_events (id, contract_id, event_type, description, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&release_event.id)
        .bind(&release_event.contract_id)
        .bind(&release_event.event_type)
        .bind(&release_event.description)
        .bind(&release_event.created_at)
        .execute(pool.as_ref())
        .await?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message":    "Assinatura registrada",
        "role":       role,
        "sig_count":  sig_count + 1,
        "released":   sig_count >= 2,
    })))
}

// ─── POST /api/contracts/:id/dispute ─────────────────────────
async fn dispute_contract(
    pool: web::Data<SqlitePool>,
    req:  HttpRequest,
    path: web::Path<String>,
    body: web::Json<DisputeRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let id = path.into_inner();

    // Buscar contrato
    let contract = sqlx::query_as::<_, Contract>(
        "SELECT * FROM contracts WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Contrato não encontrado".into()))?;

    // Só pode abrir disputa se estiver LOCKED
    if contract.state != ContractState::Locked.as_str() {
        return Err(AppError::InvalidContractState(
            "Só é possível abrir disputa em contratos LOCKED".into(),
        ));
    }

    // Verificar se é parte do contrato
    let signer_pubkey = sqlx::query_scalar::<_, String>(
        "SELECT pubkey FROM wallets WHERE address = ?",
    )
    .bind(&claims.address)
    .fetch_one(pool.as_ref())
    .await?;

    if signer_pubkey != contract.buyer_pubkey && signer_pubkey != contract.seller_pubkey {
        return Err(AppError::Unauthorized);
    }

    let now = chrono::Utc::now().to_rfc3339();

    // Atualizar estado para DISPUTED
    sqlx::query(
        "UPDATE contracts SET state = 'DISPUTED', updated_at = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(&id)
    .execute(pool.as_ref())
    .await?;

    // Registrar evento DISPUTED
    let event = ContractEvent::new(
        id.clone(),
        "DISPUTED".into(),
        Some(format!("Disputa aberta: {}", body.reason)),
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
    .execute(pool.as_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Disputa aberta. O árbitro irá analisar o caso.",
        "contract_id": id,
        "state": "DISPUTED",
    })))
}