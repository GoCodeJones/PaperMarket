use secp256k1::{Secp256k1, SecretKey, PublicKey};
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};

use crate::crypto::bip39::mnemonic_to_seed;

type HmacSha256 = Hmac<Sha256>;

// ─── Derivar par de chaves a partir do mnemônico ─────────────
pub fn derive_keypair(
    mnemonic:   &[String],
    passphrase: &str,
) -> Result<(String, String), String> {

    // 1. Gerar seed a partir das 12 palavras
    let seed = mnemonic_to_seed(mnemonic, passphrase);

    // 2. Derivar chave privada via HMAC-SHA256
    let mut mac = HmacSha256::new_from_slice(b"PaperMarket seed")
        .map_err(|e| e.to_string())?;
    mac.update(&seed);
    let result = mac.finalize().into_bytes();

    // 3. Criar chave privada secp256k1
    let secret_key = SecretKey::from_slice(&result[..32])
        .map_err(|e| e.to_string())?;

    // 4. Derivar chave pública
    let secp      = Secp256k1::new();
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    // 5. Serializar chaves em hex
    let pubkey_hex = hex::encode(public_key.serialize());

    // 6. Derivar endereço BPC a partir da chave pública
    let address = pubkey_to_address(&pubkey_hex);

    Ok((pubkey_hex, address))
}

// ─── Derivar endereço BPC a partir da chave pública ──────────
// Simulação do processo Bitcoin:
// SHA256(pubkey) → pegar primeiros 20 bytes → encode em hex → prefixo "1BPC"
pub fn pubkey_to_address(pubkey_hex: &str) -> String {
    let pubkey_bytes = hex::decode(pubkey_hex)
        .unwrap_or_default();

    // SHA256 da chave pública
    let mut hasher = Sha256::new();
    hasher.update(&pubkey_bytes);
    let hash = hasher.finalize();

    // Pegar primeiros 8 bytes e encodar em hex (endereço curto)
    let short = hex::encode(&hash[..8]);

    format!("1BPC{}", short.to_uppercase())
}

// ─── Verificar se um endereço é válido ───────────────────────
pub fn is_valid_address(address: &str) -> bool {
    address.starts_with("1BPC") && address.len() == 20
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::bip39::generate_mnemonic;

    #[test]
    fn test_derive_keypair() {
        let mnemonic = generate_mnemonic();
        let result = derive_keypair(&mnemonic, "test");
        assert!(result.is_ok());

        let (pubkey, address) = result.unwrap();
        assert!(!pubkey.is_empty());
        assert!(address.starts_with("1BPC"));
    }

    #[test]
    fn test_derive_keypair_deterministic() {
        let mnemonic = vec![
            "abandon".to_string(), "ability".to_string(), "able".to_string(),
            "about".to_string(), "above".to_string(), "absent".to_string(),
            "absorb".to_string(), "abstract".to_string(), "absurd".to_string(),
            "abuse".to_string(), "access".to_string(), "accident".to_string(),
        ];

        let (pubkey1, addr1) = derive_keypair(&mnemonic, "test").unwrap();
        let (pubkey2, addr2) = derive_keypair(&mnemonic, "test").unwrap();

        // Mesmas palavras + mesma passphrase → mesmo resultado
        assert_eq!(pubkey1, pubkey2);
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_is_valid_address() {
        assert!(is_valid_address("1BPCA1B2C3D4E5F6"));
        assert!(!is_valid_address("invalid"));
        assert!(!is_valid_address("2BPCinvalid"));
    }
}