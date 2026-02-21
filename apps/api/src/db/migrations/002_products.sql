-- ============================================================
-- MIGRATION 002 — Produtos e Avaliações
-- ============================================================

CREATE TABLE IF NOT EXISTS products (
    id          TEXT PRIMARY KEY,           -- UUID v4
    seller_id   TEXT NOT NULL,              -- FK → users.id
    title       TEXT NOT NULL,              -- título do produto
    description TEXT NOT NULL,              -- descrição detalhada
    price_sats  INTEGER NOT NULL,           -- preço em satoshis de BPC
    category    TEXT NOT NULL,              -- Tech, Livros, Foto, Casa, Moda...
    condition   TEXT NOT NULL,              -- Novo | Usado
    location    TEXT NOT NULL,              -- cidade/estado do vendedor
    status      TEXT NOT NULL DEFAULT 'active', -- active | sold | removed
    created_at  TEXT NOT NULL,              -- ISO 8601
    updated_at  TEXT NOT NULL,              -- ISO 8601

    FOREIGN KEY (seller_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS reviews (
    id          TEXT PRIMARY KEY,           -- UUID v4
    product_id  TEXT NOT NULL,              -- FK → products.id
    reviewer_id TEXT NOT NULL,              -- FK → users.id
    rating      INTEGER NOT NULL,           -- 1 a 5
    comment     TEXT,                       -- comentário opcional
    created_at  TEXT NOT NULL,              -- ISO 8601

    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    FOREIGN KEY (reviewer_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Índices
CREATE INDEX IF NOT EXISTS idx_products_seller_id  ON products(seller_id);
CREATE INDEX IF NOT EXISTS idx_products_category   ON products(category);
CREATE INDEX IF NOT EXISTS idx_products_status     ON products(status);
CREATE INDEX IF NOT EXISTS idx_reviews_product_id  ON reviews(product_id);