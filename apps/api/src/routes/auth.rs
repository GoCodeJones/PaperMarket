use actix_web::{web, HttpResponse};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::SqlitePool;
use std::env;
use chrono::Utc;

use crate::models::user::{
    Claims, LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, User, Wallet,
};
use crate::crypto::bip39::generate_mnemonic;
use crate::crypto::keys::derive_keypair;
use crate::errors::AppError;

// ─── Configuração das rotas ──────────────────────────────────
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login",    web::post().to(login)),
    );
}

// ─── POST /api/auth/register ─────────────────────────────────
async fn register(
    pool: web::Data<SqlitePool>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {

    // 1. Verificar se username já existe
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE username = ?",
    )
    .bind(&body.username)
    .fetch_one(pool.as_ref())
    .await?;

    if exists > 0 {
        return Err(AppError::AlreadyExists("Username já está em uso".into()));
    }

    // 2. Gerar hash da masterkey com Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(body.masterkey.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_string();

    // 3. Gerar 12 palavras (mnemônico BIP-39 simulado)
    let mnemonic = generate_mnemonic();

    // 4. Derivar par de chaves a partir do mnemônico + masterkey
    let (pubkey, address) = derive_keypair(&mnemonic, &body.masterkey)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // 5. Verificar se endereço já existe
    let addr_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM wallets WHERE address = ?",
    )
    .bind(&address)
    .fetch_one(pool.as_ref())
    .await?;

    if addr_exists > 0 {
        return Err(AppError::AlreadyExists("Endereço já existe".into()));
    }

    // 6. Salvar usuário no banco
    let user = User::new(body.username.clone(), password_hash);
    sqlx::query(
        "INSERT INTO users (id, username, password, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.password)
    .bind(&user.created_at)
    .execute(pool.as_ref())
    .await?;

    // 7. Salvar carteira no banco
    let wallet = Wallet::new(user.id.clone(), address.clone(), pubkey.clone());
    sqlx::query(
        "INSERT INTO wallets (id, user_id, address, pubkey, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&wallet.id)
    .bind(&wallet.user_id)
    .bind(&wallet.address)
    .bind(&wallet.pubkey)
    .bind(&wallet.created_at)
    .execute(pool.as_ref())
    .await?;

    // 8. Gerar JWT
    let token = generate_token(&user.id, &user.username, &address)?;

    Ok(HttpResponse::Created().json(RegisterResponse {
        token,
        address,
        pubkey,
        mnemonic, // exibido UMA vez — nunca armazenado no servidor
    }))
}

// ─── POST /api/auth/login ────────────────────────────────────
async fn login(
    pool: web::Data<SqlitePool>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {

    // 1. Buscar usuário
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ?",
    )
    .bind(&body.username)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or(AppError::InvalidCredentials)?;

    // 2. Verificar masterkey
    let parsed_hash = PasswordHash::new(&user.password)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Argon2::default()
        .verify_password(body.masterkey.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::InvalidCredentials)?;

    // 3. Buscar carteira
    let wallet = sqlx::query_as::<_, Wallet>(
        "SELECT * FROM wallets WHERE user_id = ?",
    )
    .bind(&user.id)
    .fetch_one(pool.as_ref())
    .await?;

    // 4. Gerar JWT
    let token = generate_token(&user.id, &user.username, &wallet.address)?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        token,
        address: wallet.address,
        username: user.username,
    }))
}

// ─── Gerar JWT ───────────────────────────────────────────────
fn generate_token(
    user_id:  &str,
    username: &str,
    address:  &str,
) -> Result<String, AppError> {
    let expiry_hours: i64 = env::var("JWT_EXPIRY_HOURS")
        .unwrap_or_else(|_| "72".into())
        .parse()
        .unwrap_or(72);

    let claims = Claims {
        sub:      user_id.to_string(),
        username: username.to_string(),
        address:  address.to_string(),
        exp:      (Utc::now().timestamp() + expiry_hours * 3600) as usize,
    };

    let secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "secret".into());

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}