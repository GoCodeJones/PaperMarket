-- ============================================================
-- MIGRATION 001 — Usuários e Carteiras
-- ============================================================

CREATE TABLE IF NOT EXISTS users (
    id          TEXT PRIMARY KEY,           -- UUID v4
    username    TEXT NOT NULL UNIQUE,       -- nome público do usuário
    password    TEXT NOT NULL,              -- hash Argon2 da masterkey
    created_at  TEXT NOT NULL               -- ISO 8601
);

CREATE TABLE IF NOT EXISTS wallets (
    id          TEXT PRIMARY KEY,           -- UUID v4
    user_id     TEXT NOT NULL UNIQUE,       -- FK → users.id
    address     TEXT NOT NULL UNIQUE,       -- endereço BPC (ex: 1BPC...f4a9)
    pubkey      TEXT NOT NULL UNIQUE,       -- chave pública secp256k1 (hex)
    created_at  TEXT NOT NULL,              -- ISO 8601

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Índices
CREATE INDEX IF NOT EXISTS idx_wallets_address ON wallets(address);
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);