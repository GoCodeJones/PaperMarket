use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::Utc;
use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::user::Claims;
use crate::models::product::{
    CreateProductRequest, CreateReviewRequest, Product,
    ProductFilters, ProductResponse, Review, UpdateProductRequest,
};

// ─── Configuração das rotas ──────────────────────────────────
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/marketplace")
            .route("/products",              web::get().to(list_products))
            .route("/products",              web::post().to(create_product))
            .route("/products/{id}",         web::get().to(get_product))
            .route("/products/{id}",         web::put().to(update_product))
            .route("/products/{id}",         web::delete().to(delete_product))
            .route("/products/{id}/reviews", web::post().to(create_review)),
    );
}

// ─── GET /api/marketplace/products ──────────────────────────
async fn list_products(
    pool:    web::Data<SqlitePool>,
    filters: web::Query<ProductFilters>,
) -> Result<HttpResponse, AppError> {
    let page  = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    // Montar query dinâmica com filtros
    let mut conditions = vec!["p.status = 'active'"];
    let mut query = String::from(
        "SELECT p.*, u.username as seller_username,
                AVG(r.rating) as avg_rating,
                COUNT(r.id) as review_count
         FROM products p
         LEFT JOIN users u ON p.seller_id = u.id
         LEFT JOIN reviews r ON p.id = r.product_id
         WHERE p.status = 'active'"
    );

    if filters.category.is_some() {
        query.push_str(" AND p.category = ?");
    }
    if filters.condition.is_some() {
        query.push_str(" AND p.condition = ?");
    }
    if filters.min_price.is_some() {
        query.push_str(" AND p.price_sats >= ?");
    }
    if filters.max_price.is_some() {
        query.push_str(" AND p.price_sats <= ?");
    }
    if filters.search.is_some() {
        query.push_str(" AND (p.title LIKE ? OR p.description LIKE ?)");
    }

    query.push_str(" GROUP BY p.id ORDER BY p.created_at DESC LIMIT ? OFFSET ?");

    // Buscar produtos
    let mut q = sqlx::query_as::<_, Product>(&query);

    if let Some(ref cat) = filters.category {
        q = q.bind(cat);
    }
    if let Some(ref cond) = filters.condition {
        q = q.bind(cond);
    }
    if let Some(min) = filters.min_price {
        q = q.bind(min);
    }
    if let Some(max) = filters.max_price {
        q = q.bind(max);
    }
    if let Some(ref search) = filters.search {
        let pattern = format!("%{}%", search);
        q = q.bind(pattern.clone()).bind(pattern);
    }

    q = q.bind(limit).bind(offset);

    let products = q.fetch_all(pool.as_ref()).await?;

    Ok(HttpResponse::Ok().json(products))
}

// ─── GET /api/marketplace/products/:id ──────────────────────
async fn get_product(
    pool: web::Data<SqlitePool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let product = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Produto não encontrado".into()))?;

    // Buscar username do vendedor
    let seller = sqlx::query_scalar::<_, String>(
        "SELECT username FROM users WHERE id = ?",
    )
    .bind(&product.seller_id)
    .fetch_one(pool.as_ref())
    .await?;

    // Buscar média de avaliações
    let avg_rating = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT AVG(rating) FROM reviews WHERE product_id = ?",
    )
    .bind(&id)
    .fetch_one(pool.as_ref())
    .await?;

    let review_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM reviews WHERE product_id = ?",
    )
    .bind(&id)
    .fetch_one(pool.as_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ProductResponse {
        product,
        seller,
        avg_rating,
        review_count,
    }))
}

// ─── POST /api/marketplace/products ─────────────────────────
async fn create_product(
    pool: web::Data<SqlitePool>,
    req:  HttpRequest,
    body: web::Json<CreateProductRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    // Validações
    if body.title.trim().is_empty() {
        return Err(AppError::Validation("Título é obrigatório".into()));
    }
    if body.price_sats <= 0 {
        return Err(AppError::Validation("Preço deve ser maior que zero".into()));
    }
    if !["Novo", "Usado"].contains(&body.condition.as_str()) {
        return Err(AppError::Validation("Condição deve ser 'Novo' ou 'Usado'".into()));
    }

    // Buscar user_id a partir do endereço
    let user_id = sqlx::query_scalar::<_, String>(
        "SELECT user_id FROM wallets WHERE address = ?",
    )
    .bind(&claims.address)
    .fetch_one(pool.as_ref())
    .await?;

    let product = Product::new(
        user_id,
        body.title.clone(),
        body.description.clone(),
        body.price_sats,
        body.category.clone(),
        body.condition.clone(),
        body.location.clone(),
    );

    sqlx::query(
        "INSERT INTO products (id, seller_id, title, description, price_sats, category, condition, location, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&product.id)
    .bind(&product.seller_id)
    .bind(&product.title)
    .bind(&product.description)
    .bind(product.price_sats)
    .bind(&product.category)
    .bind(&product.condition)
    .bind(&product.location)
    .bind(&product.status)
    .bind(&product.created_at)
    .bind(&product.updated_at)
    .execute(pool.as_ref())
    .await?;

    Ok(HttpResponse::Created().json(product))
}

// ─── PUT /api/marketplace/products/:id ──────────────────────
async fn update_product(
    pool: web::Data<SqlitePool>,
    req:  HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateProductRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let id = path.into_inner();

    // Verificar se produto existe e pertence ao usuário
    let user_id = sqlx::query_scalar::<_, String>(
        "SELECT user_id FROM wallets WHERE address = ?",
    )
    .bind(&claims.address)
    .fetch_one(pool.as_ref())
    .await?;

    let owner_id = sqlx::query_scalar::<_, String>(
        "SELECT seller_id FROM products WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Produto não encontrado".into()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized);
    }

    let now = Utc::now().to_rfc3339();

    // Atualizar campos opcionais
    if let Some(ref title) = body.title {
        sqlx::query("UPDATE products SET title = ?, updated_at = ? WHERE id = ?")
            .bind(title).bind(&now).bind(&id)
            .execute(pool.as_ref()).await?;
    }
    if let Some(ref desc) = body.description {
        sqlx::query("UPDATE products SET description = ?, updated_at = ? WHERE id = ?")
            .bind(desc).bind(&now).bind(&id)
            .execute(pool.as_ref()).await?;
    }
    if let Some(price) = body.price_sats {
        sqlx::query("UPDATE products SET price_sats = ?, updated_at = ? WHERE id = ?")
            .bind(price).bind(&now).bind(&id)
            .execute(pool.as_ref()).await?;
    }
    if let Some(ref status) = body.status {
        sqlx::query("UPDATE products SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status).bind(&now).bind(&id)
            .execute(pool.as_ref()).await?;
    }

    let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(HttpResponse::Ok().json(product))
}

// ─── DELETE /api/marketplace/products/:id ───────────────────
async fn delete_product(
    pool: web::Data<SqlitePool>,
    req:  HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let id = path.into_inner();

    let user_id = sqlx::query_scalar::<_, String>(
        "SELECT user_id FROM wallets WHERE address = ?",
    )
    .bind(&claims.address)
    .fetch_one(pool.as_ref())
    .await?;

    let owner_id = sqlx::query_scalar::<_, String>(
        "SELECT seller_id FROM products WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Produto não encontrado".into()))?;

    if owner_id != user_id {
        return Err(AppError::Unauthorized);
    }

    // Soft delete — muda status para 'removed'
    sqlx::query("UPDATE products SET status = 'removed', updated_at = ? WHERE id = ?")
        .bind(Utc::now().to_rfc3339())
        .bind(&id)
        .execute(pool.as_ref())
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Produto removido" })))
}

// ─── POST /api/marketplace/products/:id/reviews ─────────────
async fn create_review(
    pool: web::Data<SqlitePool>,
    req:  HttpRequest,
    path: web::Path<String>,
    body: web::Json<CreateReviewRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let product_id = path.into_inner();

    // Validar rating
    if body.rating < 1 || body.rating > 5 {
        return Err(AppError::Validation("Rating deve ser entre 1 e 5".into()));
    }

    // Buscar reviewer_id
    let reviewer_id = sqlx::query_scalar::<_, String>(
        "SELECT user_id FROM wallets WHERE address = ?",
    )
    .bind(&claims.address)
    .fetch_one(pool.as_ref())
    .await?;

    // Verificar se produto existe
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM products WHERE id = ?",
    )
    .bind(&product_id)
    .fetch_one(pool.as_ref())
    .await?;

    if exists == 0 {
        return Err(AppError::NotFound("Produto não encontrado".into()));
    }

    // Verificar se já avaliou
    let already_reviewed = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM reviews WHERE product_id = ? AND reviewer_id = ?",
    )
    .bind(&product_id)
    .bind(&reviewer_id)
    .fetch_one(pool.as_ref())
    .await?;

    if already_reviewed > 0 {
        return Err(AppError::AlreadyExists("Você já avaliou este produto".into()));
    }

    let review = Review::new(
        product_id,
        reviewer_id,
        body.rating,
        body.comment.clone(),
    );

    sqlx::query(
        "INSERT INTO reviews (id, product_id, reviewer_id, rating, comment, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&review.id)
    .bind(&review.product_id)
    .bind(&review.reviewer_id)
    .bind(review.rating)
    .bind(&review.comment)
    .bind(&review.created_at)
    .execute(pool.as_ref())
    .await?;

    Ok(HttpResponse::Created().json(review))
}