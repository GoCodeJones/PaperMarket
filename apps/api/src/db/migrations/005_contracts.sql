-- ============================================================
-- MIGRATION 005 — Smart Contracts (Escrow)
-- ============================================================

CREATE TABLE IF NOT EXISTS contracts (
    id                  TEXT PRIMARY KEY,   -- UUID v4
    version             TEXT NOT NULL DEFAULT 'CONTRACT_ESCROW_v1',
    product_id          TEXT NOT NULL,      -- FK → products.id
    buyer_pubkey        TEXT NOT NULL,      -- chave pública do comprador
    seller_pubkey       TEXT NOT NULL,      -- chave pública do vendedor
    arbiter_pubkey      TEXT NOT NULL,      -- chave pública do árbitro
    amount_sats         INTEGER NOT NULL,   -- valor bloqueado em satoshis
    fee_sats            INTEGER NOT NULL,   -- taxa do escrow em satoshis
    item_hash           TEXT NOT NULL,      -- SHA-256 da descrição do produto
    state               TEXT NOT NULL DEFAULT 'PENDING',
                                            -- PENDING | LOCKED | RELEASED
                                            -- DISPUTED | REFUNDED
    created_at_block    INTEGER NOT NULL,   -- altura do bloco de criação
    expires_at_block    INTEGER NOT NULL,   -- altura do bloco de expiração
    lock_tx_id          TEXT,               -- FK → transactions.id (TX de bloqueio)
    release_tx_id       TEXT,               -- FK → transactions.id (TX de liberação)
    created_at          TEXT NOT NULL,      -- ISO 8601
    updated_at          TEXT NOT NULL,      -- ISO 8601

    FOREIGN KEY (product_id)   REFERENCES products(id)      ON DELETE RESTRICT,
    FOREIGN KEY (lock_tx_id)   REFERENCES transactions(id)  ON DELETE SET NULL,
    FOREIGN KEY (release_tx_id) REFERENCES transactions(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS contract_signatures (
    id              TEXT PRIMARY KEY,       -- UUID v4
    contract_id     TEXT NOT NULL,          -- FK → contracts.id
    signer_pubkey   TEXT NOT NULL,          -- quem assinou
    signature       TEXT NOT NULL,          -- assinatura secp256k1 (hex)
    role            TEXT NOT NULL,          -- buyer | seller | arbiter
    signed_at       TEXT NOT NULL,          -- ISO 8601

    FOREIGN KEY (contract_id) REFERENCES contracts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS contract_events (
    id              TEXT PRIMARY KEY,       -- UUID v4
    contract_id     TEXT NOT NULL,          -- FK → contracts.id
    event_type      TEXT NOT NULL,          -- CREATED | LOCKED | SIGNED
                                            -- DISPUTED | RELEASED | REFUNDED
    description     TEXT,                   -- detalhes do evento
    created_at      TEXT NOT NULL,          -- ISO 8601

    FOREIGN KEY (contract_id) REFERENCES contracts(id) ON DELETE CASCADE
);

-- Índices
CREATE INDEX IF NOT EXISTS idx_contracts_state          ON contracts(state);
CREATE INDEX IF NOT EXISTS idx_contracts_product_id     ON contracts(product_id);
CREATE INDEX IF NOT EXISTS idx_contracts_buyer_pubkey   ON contracts(buyer_pubkey);
CREATE INDEX IF NOT EXISTS idx_contracts_seller_pubkey  ON contracts(seller_pubkey);
CREATE INDEX IF NOT EXISTS idx_contract_signatures_contract_id ON contract_signatures(contract_id);
CREATE INDEX IF NOT EXISTS idx_contract_events_contract_id     ON contract_events(contract_id);