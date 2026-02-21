use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Usuário ─────────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id:         String,
    pub username:   String,
    pub password:   String,    // hash Argon2 — nunca expor na API
    pub created_at: String,
}

impl User {
    pub fn new(username: String, password_hash: String) -> Self {
        Self {
            id:         Uuid::new_v4().to_string(),
            username,
            password:   password_hash,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

// ─── Carteira ────────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Wallet {
    pub id:         String,
    pub user_id:    String,
    pub address:    String,    // endereço BPC (ex: 1BPC...f4a9)
    pub pubkey:     String,    // chave pública secp256k1 (hex)
    pub created_at: String,
}

impl Wallet {
    pub fn new(user_id: String, address: String, pubkey: String) -> Self {
        Self {
            id:         Uuid::new_v4().to_string(),
            user_id,
            address,
            pubkey,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

// ─── DTOs ────────────────────────────────────────────────────

/// Payload de registro
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username:   String,
    pub masterkey:  String,    // senha do usuário — derivada em chave privada
}

/// Payload de login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username:   String,
    pub masterkey:  String,
}

/// Resposta de registro — inclui as 12 palavras (exibidas UMA vez)
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub token:      String,    // JWT
    pub address:    String,    // endereço BPC
    pub pubkey:     String,    // chave pública
    pub mnemonic:   Vec<String>, // 12 palavras — nunca armazenadas no servidor
}

/// Resposta de login
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token:      String,    // JWT
    pub address:    String,    // endereço BPC
    pub username:   String,
}

/// Claims do JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub:        String,    // user id
    pub username:   String,
    pub address:    String,    // endereço BPC
    pub exp:        usize,     // expiração (timestamp Unix)
}