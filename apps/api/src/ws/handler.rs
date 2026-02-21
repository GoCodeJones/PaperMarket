use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;
use sqlx::SqlitePool;
use tokio::time::{interval, Duration};
use tracing::info;

use crate::blockchain::mempool::{mempool_count, average_fee};
use crate::blockchain::chain::get_latest_block;

// ─── Módulo WebSocket ────────────────────────────────────────
pub mod ws {
    pub mod handler;
}

// ─── Handler principal do WebSocket ──────────────────────────
pub async fn ws_handler(
    req:  HttpRequest,
    body: web::Payload,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let pool = pool.into_inner();

    actix_web::rt::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                // ─── Tick a cada 5 segundos ──────────────────
                _ = ticker.tick() => {
                    // Buscar estado atual da chain
                    let chain_state = get_chain_state(&pool).await;

                    let msg = serde_json::json!({
                        "type": "chain_update",
                        "data": chain_state,
                    });

                    if session
                        .text(msg.to_string())
                        .await
                        .is_err()
                    {
                        break;
                    }
                }

                // ─── Mensagens do cliente ────────────────────
                Some(Ok(msg)) = msg_stream.recv() => {
                    match msg {
                        Message::Text(text) => {
                            handle_client_message(
                                &mut session,
                                &pool,
                                &text,
                            ).await;
                        }
                        Message::Ping(bytes) => {
                            let _ = session.pong(&bytes).await;
                        }
                        Message::Close(_) => {
                            info!("WebSocket: cliente desconectou");
                            break;
                        }
                        _ => {}
                    }
                }

                else => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}

// ─── Processar mensagens do cliente ──────────────────────────
async fn handle_client_message(
    session: &mut actix_ws::Session,
    pool:    &SqlitePool,
    text:    &str,
) {
    let Ok(msg) = serde_json::from_str::<serde_json::Value>(text) else {
        let _ = session.text(serde_json::json!({
            "type":    "error",
            "message": "Mensagem inválida",
        }).to_string()).await;
        return;
    };

    let event_type = msg["type"].as_str().unwrap_or("");

    match event_type {
        // Cliente pede estado atual da chain
        "get_chain" => {
            let data = get_chain_state(pool).await;
            let _ = session.text(serde_json::json!({
                "type": "chain_update",
                "data": data,
            }).to_string()).await;
        }

        // Cliente pede info da mempool
        "get_mempool" => {
            let count = mempool_count(pool).await.unwrap_or(0);
            let avg   = average_fee(pool).await.unwrap_or(0);

            let _ = session.text(serde_json::json!({
                "type": "mempool_update",
                "data": {
                    "count":   count,
                    "avg_fee": avg,
                },
            }).to_string()).await;
        }

        // Ping manual
        "ping" => {
            let _ = session.text(serde_json::json!({
                "type": "pong",
            }).to_string()).await;
        }

        _ => {
            let _ = session.text(serde_json::json!({
                "type":    "error",
                "message": format!("Evento desconhecido: {}", event_type),
            }).to_string()).await;
        }
    }
}

// ─── Montar estado atual da chain ────────────────────────────
async fn get_chain_state(pool: &SqlitePool) -> serde_json::Value {
    let (best_hash, height, difficulty) = get_latest_block(pool)
        .await
        .unwrap_or_else(|_| ("0000".into(), 0, 4));

    let mempool = mempool_count(pool).await.unwrap_or(0);
    let avg_fee = average_fee(pool).await.unwrap_or(0);

    serde_json::json!({
        "height":     height,
        "best_hash":  best_hash,
        "difficulty": difficulty,
        "mempool":    mempool,
        "avg_fee":    avg_fee,
    })
}