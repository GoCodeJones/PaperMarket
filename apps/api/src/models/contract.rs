use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Estado do contrato ──────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ContractState {
    Pending,    // criado, aguardando bloqueio de fundos
    Locked,     // fundos bloqueados na chain
    Released,   // fundos liberados ao vendedor
    Disputed,   // disputa aberta
    Refunded,   // fundos devolvidos ao comprador
}

impl ContractState {
    pub fn as_str(&self) -> &str {
        match self {
            ContractState::Pending  => "PENDING",
            ContractState::Locked   => "LOCKED",
            ContractState::Released => "RELEASED",
            ContractState::Disputed => "DISPUTED",
            ContractState::Refunded => "REFUNDED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "LOCKED"   => ContractState::Locked,
            "RELEASED" => ContractState::Released,
            "DISPUTED" => ContractState::Disputed,
            "REFUNDED" => ContractState::Refunded,
            _          => ContractState::Pending,
        }
    }
}

// ─── Contrato Escrow ─────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Contract {
    pub id:               String,
    pub version:          String,   // CONTRACT_ESCROW_v1
    pub product_id:       String,
    pub buyer_pubkey:     String,
    pub seller_pubkey:    String,
    pub arbiter_pubkey:   String,
    pub amount_sats:      i64,
    pub fee_sats:         i64,
    pub item_hash:        String,   // SHA-256 da descrição do produto
    pub state:            String,   // ContractState serializado
    pub created_at_block: i64,
    pub expires_at_block: i64,
    pub lock_tx_id:       Option<String>,
    pub release_tx_id:    Option<String>,
    pub created_at:       String,
    pub updated_at:       String,
}

impl Contract {
    pub fn new(
        product_id:       String,
        buyer_pubkey:     String,
        seller_pubkey:    String,
        arbiter_pubkey:   String,
        amount_sats:      i64,
        fee_sats:         i64,
        item_hash:        String,
        created_at_block: i64,
        expires_at_block: i64,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id:               Uuid::new_v4().to_string(),
            version:          "CONTRACT_ESCROW_v1".into(),
            product_id,
            buyer_pubkey,
            seller_pubkey,
            arbiter_pubkey,
            amount_sats,
            fee_sats,
            item_hash,
            state:            ContractState::Pending.as_str().into(),
            created_at_block,
            expires_at_block,
            lock_tx_id:       None,
            release_tx_id:    None,
            created_at:       now.clone(),
            updated_at:       now,
        }
    }
}

// ─── Assinatura do contrato ──────────────────────────────────
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContractSignature {
    pub id:            String,
    pub contract_id:   String,
    pub signer_pubkey: String,
    pub signature:     String,   // assinatura secp256k1 (hex)
    pub role:          String,   // buyer | seller | arbiter
    pub signed_at:     String,
}

impl ContractSignature {
    pub fn new(
        contract_id:   String,
        signer_pubkey: String,
        signature:     String,
        role:          String,
    ) -> Self {
        Self {
            id:            Uuid::new_v4().to_string(),
            contract_id,
            signer_pubkey,
            signature,
            role,
            signed_at:     Utc::now().to_rfc3339(),
        }
    }
}

// ─── Evento do contrato ──────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContractEvent {
    pub id:          String,
    pub contract_id: String,
    pub event_type:  String,   // CREATED | LOCKED | SIGNED | DISPUTED | RELEASED | REFUNDED
    pub description: Option<String>,
    pub created_at:  String,
}

impl ContractEvent {
    pub fn new(
        contract_id: String,
        event_type:  String,
        description: Option<String>,
    ) -> Self {
        Self {
            id:          Uuid::new_v4().to_string(),
            contract_id,
            event_type,
            description,
            created_at:  Utc::now().to_rfc3339(),
        }
    }
}

// ─── DTOs ────────────────────────────────────────────────────

/// Criar contrato escrow
#[derive(Debug, Deserialize)]
pub struct CreateEscrowRequest {
    pub product_id:      String,
    pub seller_pubkey:   String,
    pub amount_sats:     i64,
}

/// Assinar contrato
#[derive(Debug, Deserialize)]
pub struct SignContractRequest {
    pub signature:       String,   // assinatura secp256k1 do comprador/vendedor/árbitro
}

/// Abrir disputa
#[derive(Debug, Deserialize)]
pub struct DisputeRequest {
    pub reason:          String,
}

/// Resposta completa do contrato
#[derive(Debug, Serialize)]
pub struct ContractResponse {
    pub contract:    Contract,
    pub signatures:  Vec<ContractSignature>,
    pub events:      Vec<ContractEvent>,
    pub signed_by:   Vec<String>,   // roles que já assinaram
    pub can_release: bool,          // true se tiver 2/3 assinaturas
}