use sha2::{Digest, Sha256};

// ─── SHA-256 hex string ───────────────────────────────────────
pub fn sha256_hex(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

// ─── Verificar se hash atende à dificuldade ───────────────────
// Dificuldade = quantidade de zeros no início do hash
pub fn meets_difficulty(hash: &str, difficulty: i64) -> bool {
    let target = "0".repeat(difficulty as usize);
    hash.starts_with(&target)
}

// ─── Minerar um bloco (usado em testes e genesis) ────────────
pub fn mine_block(
    height:      i64,
    prev_hash:   &str,
    merkle_root: &str,
    difficulty:  i64,
    timestamp:   &str,
) -> (i64, String) {
    let mut nonce: i64 = 0;

    loop {
        let data = format!(
            "{}{}{}{}{}{}",
            height, prev_hash, merkle_root, nonce, difficulty, timestamp
        );
        let hash = sha256_hex(&data);

        if meets_difficulty(&hash, difficulty) {
            return (nonce, hash);
        }

        nonce += 1;
    }
}

// ─── Calcular target em formato legível ───────────────────────
pub fn difficulty_to_target(difficulty: i64) -> String {
    format!("{}{}",
        "0".repeat(difficulty as usize),
        "f".repeat(64 - difficulty as usize),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex("hello");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_meets_difficulty() {
        assert!(meets_difficulty("0000abcdef", 4));
        assert!(!meets_difficulty("000abcdef0", 4));
        assert!(meets_difficulty("00abcdef00", 2));
    }

    #[test]
    fn test_mine_block_difficulty_1() {
        let (nonce, hash) = mine_block(1, "0000", "abc", 1, "2024-01-01");
        assert!(meets_difficulty(&hash, 1));
        assert!(nonce >= 0);
    }

    #[test]
    fn test_difficulty_to_target() {
        let target = difficulty_to_target(4);
        assert!(target.starts_with("0000"));
        assert_eq!(target.len(), 64);
    }
}