use chrono::Utc;
use serde::{Deserialize, Serialize};

// ─── Bloco ───────────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Block {
    pub id:             String,    // hash SHA-256 do bloco
    pub height:         i64,       // altura na chain
    pub prev_hash:      String,    // hash do bloco anterior
    pub merkle_root:    String,    // merkle root das TXs
    pub nonce:          i64,       // nonce encontrado no PoW
    pub difficulty:     i64,       // dificuldade no momento
    pub reward_sats:    i64,       // recompensa em satoshis
    pub miner_address:  String,    // endereço BPC do minerador
    pub tx_count:       i64,       // quantidade de TXs
    pub mined_at:       String,    // ISO 8601
}

impl Block {
    pub fn new(
        id:            String,
        height:        i64,
        prev_hash:     String,
        merkle_root:   String,
        nonce:         i64,
        difficulty:    i64,
        reward_sats:   i64,
        miner_address: String,
        tx_count:      i64,
    ) -> Self {
        Self {
            id,
            height,
            prev_hash,
            merkle_root,
            nonce,
            difficulty,
            reward_sats,
            miner_address,
            tx_count,
            mined_at: Utc::now().to_rfc3339(),
        }
    }
}

// ─── Header do bloco (usado no PoW) ──────────────────────────
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height:      i64,
    pub prev_hash:   String,
    pub merkle_root: String,
    pub nonce:       i64,
    pub difficulty:  i64,
    pub timestamp:   String,
}

impl BlockHeader {
    pub fn new(
        height:      i64,
        prev_hash:   String,
        merkle_root: String,
        difficulty:  i64,
    ) -> Self {
        Self {
            height,
            prev_hash,
            merkle_root,
            nonce:      0,
            difficulty,
            timestamp:  Utc::now().to_rfc3339(),
        }
    }

    /// Serializa o header para hashing
    pub fn to_bytes(&self) -> String {
        format!(
            "{}{}{}{}{}{}",
            self.height,
            self.prev_hash,
            self.merkle_root,
            self.nonce,
            self.difficulty,
            self.timestamp,
        )
    }
}

// ─── DTOs ────────────────────────────────────────────────────

/// Job de mineração enviado ao minerador
#[derive(Debug, Serialize)]
pub struct MiningJob {
    pub block_height:  i64,
    pub prev_hash:     String,
    pub merkle_root:   String,
    pub difficulty:    i64,
    pub target:        String,    // string de zeros ex: "0000..."
    pub reward_sats:   i64,
}

/// Submissão de bloco minerado
#[derive(Debug, Deserialize)]
pub struct MiningSubmit {
    pub block_height:  i64,
    pub prev_hash:     String,
    pub merkle_root:   String,
    pub nonce:         i64,
    pub miner_address: String,
    pub timestamp:     String,
}

/// Resposta do block explorer
#[derive(Debug, Serialize)]
pub struct BlockResponse {
    pub block:        Block,
    pub transactions: Vec<String>,  // lista de tx_ids
}

/// Info geral da chain
#[derive(Debug, Serialize)]
pub struct ChainInfo {
    pub height:          i64,
    pub best_hash:       String,
    pub difficulty:      i64,
    pub total_supply:    i64,   // BPC emitido até agora em satoshis
    pub mempool_count:   i64,   // TXs pendentes
    pub block_reward:    i64,   // recompensa atual em satoshis
}