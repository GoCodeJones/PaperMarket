-- ============================================================
-- MIGRATION 004 — Blocos
-- ============================================================

CREATE TABLE IF NOT EXISTS blocks (
    id              TEXT PRIMARY KEY,       -- hash SHA-256 do bloco
    height          INTEGER NOT NULL UNIQUE,-- altura do bloco na chain
    prev_hash       TEXT NOT NULL,          -- hash do bloco anterior
    merkle_root     TEXT NOT NULL,          -- merkle root das TXs do bloco
    nonce           INTEGER NOT NULL,       -- nonce encontrado no PoW
    difficulty      INTEGER NOT NULL,       -- dificuldade no momento do bloco
    reward_sats     INTEGER NOT NULL,       -- recompensa em satoshis
    miner_address   TEXT NOT NULL,          -- endereço BPC do minerador
    tx_count        INTEGER NOT NULL DEFAULT 0, -- quantidade de TXs no bloco
    mined_at        TEXT NOT NULL,          -- ISO 8601

    FOREIGN KEY (miner_address) REFERENCES wallets(address)
);

-- Índices
CREATE INDEX IF NOT EXISTS idx_blocks_height        ON blocks(height);
CREATE INDEX IF NOT EXISTS idx_blocks_miner_address ON blocks(miner_address);
CREATE INDEX IF NOT EXISTS idx_blocks_mined_at      ON blocks(mined_at);