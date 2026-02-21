-- ============================================================
-- MIGRATION 003 — Transações e UTXOs
-- ============================================================

CREATE TABLE IF NOT EXISTS transactions (
    id          TEXT PRIMARY KEY,           -- hash SHA-256 da TX
    block_id    TEXT,                       -- FK → blocks.id (NULL se pendente)
    sender      TEXT NOT NULL,              -- endereço BPC remetente
    receiver    TEXT NOT NULL,              -- endereço BPC destinatário
    amount_sats INTEGER NOT NULL,           -- valor em satoshis de BPC
    fee_sats    INTEGER NOT NULL DEFAULT 0, -- taxa em satoshis
    signature   TEXT NOT NULL,              -- assinatura secp256k1 (hex)
    status      TEXT NOT NULL DEFAULT 'pending', -- pending | confirmed | rejected
    created_at  TEXT NOT NULL,              -- ISO 8601

    FOREIGN KEY (block_id) REFERENCES blocks(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS utxos (
    id          TEXT PRIMARY KEY,           -- UUID v4
    tx_id       TEXT NOT NULL,              -- FK → transactions.id
    owner       TEXT NOT NULL,              -- endereço BPC do dono
    amount_sats INTEGER NOT NULL,           -- valor em satoshis
    spent       INTEGER NOT NULL DEFAULT 0, -- 0 = não gasto | 1 = gasto
    spent_tx_id TEXT,                       -- FK → transactions.id (TX que gastou)
    created_at  TEXT NOT NULL,              -- ISO 8601

    FOREIGN KEY (tx_id) REFERENCES transactions(id) ON DELETE CASCADE
);

-- Índices
CREATE INDEX IF NOT EXISTS idx_transactions_sender    ON transactions(sender);
CREATE INDEX IF NOT EXISTS idx_transactions_receiver  ON transactions(receiver);
CREATE INDEX IF NOT EXISTS idx_transactions_status    ON transactions(status);
CREATE INDEX IF NOT EXISTS idx_transactions_block_id  ON transactions(block_id);
CREATE INDEX IF NOT EXISTS idx_utxos_owner            ON utxos(owner);
CREATE INDEX IF NOT EXISTS idx_utxos_spent            ON utxos(spent);