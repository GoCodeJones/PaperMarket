use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Transação ───────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id:          String,   // hash SHA-256 da TX
    pub block_id:    Option<String>,
    pub sender:      String,   // endereço BPC remetente
    pub receiver:    String,   // endereço BPC destinatário
    pub amount_sats: i64,      // valor em satoshis
    pub fee_sats:    i64,      // taxa em satoshis
    pub signature:   String,   // assinatura secp256k1 (hex)
    pub status:      String,   // pending | confirmed | rejected
    pub created_at:  String,
}

impl Transaction {
    pub fn new(
        id:          String,
        sender:      String,
        receiver:    String,
        amount_sats: i64,
        fee_sats:    i64,
        signature:   String,
    ) -> Self {
        Self {
            id,
            block_id:    None,
            sender,
            receiver,
            amount_sats,
            fee_sats,
            status:      "pending".into(),
            signature,
            created_at:  Utc::now().to_rfc3339(),
        }
    }
}

// ─── UTXO ────────────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Utxo {
    pub id:          String,
    pub tx_id:       String,
    pub owner:       String,   // endereço BPC do dono
    pub amount_sats: i64,
    pub spent:       i64,      // 0 = não gasto | 1 = gasto
    pub spent_tx_id: Option<String>,
    pub created_at:  String,
}

impl Utxo {
    pub fn new(tx_id: String, owner: String, amount_sats: i64) -> Self {
        Self {
            id:          Uuid::new_v4().to_string(),
            tx_id,
            owner,
            amount_sats,
            spent:       0,
            spent_tx_id: None,
            created_at:  Utc::now().to_rfc3339(),
        }
    }
}

// ─── DTOs ────────────────────────────────────────────────────

/// Enviar BPC
#[derive(Debug, Deserialize)]
pub struct SendRequest {
    pub receiver:    String,   // endereço BPC destinatário
    pub amount_sats: i64,
    pub fee_sats:    i64,
    pub signature:   String,   // assinatura da TX feita no cliente
}

/// Resposta de envio
#[derive(Debug, Serialize)]
pub struct SendResponse {
    pub tx_id:       String,
    pub status:      String,
    pub amount_sats: i64,
    pub fee_sats:    i64,
}

/// Saldo da carteira
#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub address:         String,
    pub balance_sats:    i64,   // saldo confirmado
    pub pending_sats:    i64,   // saldo pendente (mempool)
    pub utxo_count:      i64,
}

/// Histórico de transações
#[derive(Debug, Serialize)]
pub struct TxHistoryResponse {
    pub transactions: Vec<Transaction>,
    pub total:        i64,
    pub page:         i64,
}