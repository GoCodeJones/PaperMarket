use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Produto ─────────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id:          String,
    pub seller_id:   String,
    pub title:       String,
    pub description: String,
    pub price_sats:  i64,      // preço em satoshis de BPC
    pub category:    String,
    pub condition:   String,   // Novo | Usado
    pub location:    String,
    pub status:      String,   // active | sold | removed
    pub created_at:  String,
    pub updated_at:  String,
}

impl Product {
    pub fn new(
        seller_id:   String,
        title:       String,
        description: String,
        price_sats:  i64,
        category:    String,
        condition:   String,
        location:    String,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id:          Uuid::new_v4().to_string(),
            seller_id,
            title,
            description,
            price_sats,
            category,
            condition,
            location,
            status:      "active".into(),
            created_at:  now.clone(),
            updated_at:  now,
        }
    }
}

// ─── Avaliação ───────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Review {
    pub id:          String,
    pub product_id:  String,
    pub reviewer_id: String,
    pub rating:      i64,      // 1 a 5
    pub comment:     Option<String>,
    pub created_at:  String,
}

impl Review {
    pub fn new(
        product_id:  String,
        reviewer_id: String,
        rating:      i64,
        comment:     Option<String>,
    ) -> Self {
        Self {
            id:          Uuid::new_v4().to_string(),
            product_id,
            reviewer_id,
            rating,
            comment,
            created_at:  Utc::now().to_rfc3339(),
        }
    }
}

// ─── DTOs ────────────────────────────────────────────────────

/// Criar produto
#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub title:       String,
    pub description: String,
    pub price_sats:  i64,
    pub category:    String,
    pub condition:   String,
    pub location:    String,
}

/// Editar produto
#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub title:       Option<String>,
    pub description: Option<String>,
    pub price_sats:  Option<i64>,
    pub category:    Option<String>,
    pub condition:   Option<String>,
    pub location:    Option<String>,
    pub status:      Option<String>,
}

/// Filtros de listagem
#[derive(Debug, Deserialize)]
pub struct ProductFilters {
    pub category:   Option<String>,
    pub condition:  Option<String>,
    pub min_price:  Option<i64>,
    pub max_price:  Option<i64>,
    pub search:     Option<String>,
    pub page:       Option<i64>,
    pub limit:      Option<i64>,
}

/// Criar avaliação
#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub rating:  i64,
    pub comment: Option<String>,
}

/// Produto com média de avaliações (resposta da API)
#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub product:       Product,
    pub seller:        String,   // username do vendedor
    pub avg_rating:    Option<f64>,
    pub review_count:  i64,
}