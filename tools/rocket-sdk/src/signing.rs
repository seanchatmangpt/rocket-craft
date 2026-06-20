/// Ed25519 receipt signing for the cook pipeline.
///
/// The Rust CLI signs cook receipts with the private key stored in
/// ROCKET_SIGNING_KEY_PEM (base64-encoded PKCS#8 Ed25519 key). The public key
/// is stored in ROCKET_SIGNING_PUBKEY (raw base64). Both are checked into
/// nuxt-shell/.env (never into git). The browser and chain-verify endpoint use
/// the public key to verify any receipt claimed to come from the cook pipeline.
///
/// Key generation: `rocket supabase keygen` writes both keys to stdout for
/// placement in .env files.
///
/// Why Ed25519 over HMAC-SHA256:
///   HMAC requires both sides to share the secret — anyone who can verify
///   can also forge. Ed25519 is asymmetric: the CLI holds the private key
///   (can sign), the browser holds the public key (can verify, cannot forge).
use anyhow::{Context, Result};
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use ring::rand::SystemRandom;

/// Generate a new Ed25519 key pair. Returns (private_key_b64, public_key_b64).
/// Private key is PKCS#8 DER encoded; public key is raw 32-byte DER.
pub fn generate_keypair() -> Result<(String, String)> {
    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
        .map_err(|_| anyhow::anyhow!("Ed25519 key generation failed"))?;
    let keypair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
        .map_err(|_| anyhow::anyhow!("Ed25519 keypair load failed"))?;
    let private_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, pkcs8_bytes.as_ref());
    let public_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, keypair.public_key().as_ref());
    Ok((private_b64, public_b64))
}

/// Sign `message` bytes with the Ed25519 private key loaded from `private_key_b64`.
pub fn sign(private_key_b64: &str, message: &[u8]) -> Result<String> {
    let pkcs8 = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, private_key_b64)
        .context("decode private key")?;
    let keypair = Ed25519KeyPair::from_pkcs8(&pkcs8)
        .map_err(|_| anyhow::anyhow!("Ed25519 load private key failed"))?;
    let sig = keypair.sign(message);
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, sig.as_ref()))
}

/// Verify `signature_b64` over `message` using `public_key_b64`.
pub fn verify(public_key_b64: &str, message: &[u8], signature_b64: &str) -> Result<bool> {
    let pub_key_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, public_key_b64)
        .context("decode public key")?;
    let sig_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, signature_b64)
        .context("decode signature")?;
    let pk = UnparsedPublicKey::new(&ED25519, &pub_key_bytes);
    match pk.verify(message, &sig_bytes) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Build the canonical signing payload for a receipt.
/// The payload is what gets signed — same fields on both sign and verify sides.
pub fn receipt_signing_payload(
    session_id: Option<&str>,
    verdict: &str,
    receipt_hash: &str,
    proven_at: &str,
) -> Vec<u8> {
    // Sorted-key canonical JSON — same as dashboard.bak canonicalJSON()
    format!(
        "{{\"proven_at\":{},\"receipt_hash\":{},\"session_id\":{},\"verdict\":{}}}",
        serde_json::to_string(proven_at).unwrap_or_default(),
        serde_json::to_string(receipt_hash).unwrap_or_default(),
        serde_json::to_string(&session_id).unwrap_or_default(),
        serde_json::to_string(verdict).unwrap_or_default(),
    ).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keygen_produces_valid_keypair() {
        let (priv_b64, pub_b64) = generate_keypair().unwrap();
        assert!(!priv_b64.is_empty());
        assert!(!pub_b64.is_empty());
        // Public key should be 32 bytes → ~44 base64 chars
        assert!(pub_b64.len() >= 40);
    }

    #[test]
    fn sign_verify_roundtrip() {
        let (priv_b64, pub_b64) = generate_keypair().unwrap();
        let msg = b"game.receipt.verify";
        let sig = sign(&priv_b64, msg).unwrap();
        assert!(verify(&pub_b64, msg, &sig).unwrap());
    }

    #[test]
    fn tampered_message_fails_verify() {
        let (priv_b64, pub_b64) = generate_keypair().unwrap();
        let sig = sign(&priv_b64, b"original message").unwrap();
        assert!(!verify(&pub_b64, b"tampered message", &sig).unwrap());
    }

    #[test]
    fn receipt_payload_is_sorted_json() {
        let payload = receipt_signing_payload(Some("sess-1"), "PASS", "blake3:abc", "2026-01-01T00:00:00Z");
        let s = String::from_utf8(payload).unwrap();
        assert!(s.contains("\"proven_at\""));
        assert!(s.contains("\"receipt_hash\""));
        assert!(s.contains("\"verdict\""));
        // Keys must appear in alphabetical order
        let proven_pos = s.find("proven_at").unwrap();
        let receipt_pos = s.find("receipt_hash").unwrap();
        let session_pos = s.find("session_id").unwrap();
        let verdict_pos = s.find("verdict").unwrap();
        assert!(proven_pos < receipt_pos);
        assert!(receipt_pos < session_pos);
        assert!(session_pos < verdict_pos);
    }
}
