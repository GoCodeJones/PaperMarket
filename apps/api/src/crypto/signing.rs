use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
use secp256k1::ecdsa::Signature;
use sha2::{Digest, Sha256};

// ─── Assinar uma mensagem com a chave privada ─────────────────
pub fn sign_message(
    message:        &str,
    secret_key_hex: &str,
) -> Result<String, String> {
    let secp = Secp256k1::new();

    // Decodificar chave privada
    let secret_bytes = hex::decode(secret_key_hex)
        .map_err(|e| format!("Chave privada inválida: {}", e))?;

    let secret_key = SecretKey::from_slice(&secret_bytes)
        .map_err(|e| format!("Chave privada inválida: {}", e))?;

    // Hash da mensagem (SHA256)
    let msg_hash = sha256_message(message);

    let secp_msg = Message::from_digest_slice(&msg_hash)
        .map_err(|e| format!("Mensagem inválida: {}", e))?;

    // Assinar
    let signature = secp.sign_ecdsa(&secp_msg, &secret_key);

    Ok(hex::encode(signature.serialize_compact()))
}

// ─── Verificar assinatura com a chave pública ─────────────────
pub fn verify_signature(
    message:    &str,
    sig_hex:    &str,
    pubkey_hex: &str,
) -> Result<(), String> {
    let secp = Secp256k1::new();

    // Decodificar chave pública
    let pubkey_bytes = hex::decode(pubkey_hex)
        .map_err(|e| format!("Chave pública inválida: {}", e))?;

    let public_key = PublicKey::from_slice(&pubkey_bytes)
        .map_err(|e| format!("Chave pública inválida: {}", e))?;

    // Decodificar assinatura
    let sig_bytes = hex::decode(sig_hex)
        .map_err(|e| format!("Assinatura inválida: {}", e))?;

    let signature = Signature::from_compact(&sig_bytes)
        .map_err(|e| format!("Assinatura inválida: {}", e))?;

    // Hash da mensagem
    let msg_hash = sha256_message(message);

    let secp_msg = Message::from_digest_slice(&msg_hash)
        .map_err(|e| format!("Mensagem inválida: {}", e))?;

    // Verificar
    secp.verify_ecdsa(&secp_msg, &signature, &public_key)
        .map_err(|e| format!("Assinatura não confere: {}", e))?;

    Ok(())
}

// ─── SHA256 de uma mensagem → [u8; 32] ───────────────────────
fn sha256_message(message: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::{Secp256k1, SecretKey};
    use rand::rngs::OsRng;

    fn generate_test_keypair() -> (String, String) {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        (
            hex::encode(secret_key.secret_bytes()),
            hex::encode(public_key.serialize()),
        )
    }

    #[test]
    fn test_sign_and_verify() {
        let (sk, pk) = generate_test_keypair();
        let message  = "papermarket:tx:abc123";

        let signature = sign_message(message, &sk).unwrap();
        let result    = verify_signature(message, &signature, &pk);

        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_wrong_message() {
        let (sk, pk) = generate_test_keypair();
        let signature = sign_message("mensagem original", &sk).unwrap();

        let result = verify_signature("mensagem diferente", &signature, &pk);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wrong_pubkey() {
        let (sk, _)   = generate_test_keypair();
        let (_, pk2)  = generate_test_keypair();
        let message   = "papermarket:tx:abc123";

        let signature = sign_message(message, &sk).unwrap();
        let result    = verify_signature(message, &signature, &pk2);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_signature_hex() {
        let (_, pk) = generate_test_keypair();
        let result  = verify_signature("msg", "invalido", &pk);
        assert!(result.is_err());
    }
}