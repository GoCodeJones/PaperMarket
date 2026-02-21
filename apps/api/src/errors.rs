use actix_web::HttpResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // ─── Banco de dados ──────────────────────────────────────
    #[error("Erro de banco de dados: {0}")]
    Database(#[from] sqlx::Error),

    // ─── Autenticação ────────────────────────────────────────
    #[error("Credenciais inválidas")]
    InvalidCredentials,

    #[error("Token inválido ou expirado")]
    InvalidToken,

    #[error("Não autorizado")]
    Unauthorized,

    // ─── Recursos ────────────────────────────────────────────
    #[error("Recurso não encontrado: {0}")]
    NotFound(String),

    #[error("Recurso já existe: {0}")]
    AlreadyExists(String),

    // ─── Validação ───────────────────────────────────────────
    #[error("Dados inválidos: {0}")]
    Validation(String),

    // ─── Blockchain ──────────────────────────────────────────
    #[error("Saldo insuficiente")]
    InsufficientBalance,

    #[error("Transação inválida: {0}")]
    InvalidTransaction(String),

    #[error("Assinatura inválida")]
    InvalidSignature,

    #[error("Bloco inválido: {0}")]
    InvalidBlock(String),

    // ─── Contrato ────────────────────────────────────────────
    #[error("Estado do contrato inválido: {0}")]
    InvalidContractState(String),

    #[error("Contrato expirado")]
    ContractExpired,

    // ─── Genérico ────────────────────────────────────────────
    #[error("Erro interno: {0}")]
    Internal(String),
}

// ─── Resposta HTTP automática para cada erro ─────────────────
impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        use AppError::*;

        match self {
            InvalidCredentials | InvalidToken | InvalidSignature => {
                HttpResponse::Unauthorized().json(error_body(self))
            }
            Unauthorized => {
                HttpResponse::Forbidden().json(error_body(self))
            }
            NotFound(_) => {
                HttpResponse::NotFound().json(error_body(self))
            }
            AlreadyExists(_) => {
                HttpResponse::Conflict().json(error_body(self))
            }
            Validation(_) | InvalidTransaction(_) | InvalidBlock(_) | InvalidContractState(_) => {
                HttpResponse::BadRequest().json(error_body(self))
            }
            InsufficientBalance | ContractExpired => {
                HttpResponse::UnprocessableEntity().json(error_body(self))
            }
            Database(_) | Internal(_) => {
                HttpResponse::InternalServerError().json(error_body(self))
            }
        }
    }
}

// ─── Corpo padrão de erro JSON ───────────────────────────────
fn error_body(err: &AppError) -> serde_json::Value {
    serde_json::json!({
        "error": err.to_string()
    })
}