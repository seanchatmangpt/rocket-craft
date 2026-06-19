use anyhow::Result;
use colored::*;
use std::fs;
use std::path::Path;

/// Generate all missing keystores
pub fn generate_all_keystores() -> Result<()> {
    run_generate_all()
}

fn run_generate_all() -> Result<()> {
    // We will provide the commands to generate keystores using keytool,
    // as reliable native Rust PKCS12 generation is complex without heavy dependencies.

    let targets = vec![
        (
            "barbarian-road-mashines-key.keystore",
            "barbarian-road-mashines",
        ),
        ("zombie-key.keystore", "zombie"),
        ("hang3d-nightmare-keystore.keystore", "NIGHTMARE"),
    ];

    tracing::info!(
        "{}",
        "=== Android Keystore Generation Guide ===".bold().cyan()
    );
    tracing::info!("Native Rust PKCS12 generation is a work-in-progress.");
    tracing::info!("Please use the following commands to generate your signing keys:\n");

    for (name, alias) in targets {
        if Path::new(name).exists() {
            tracing::info!("{}: {}", name.yellow(), "PRESENT".green());
        } else {
            let cmd = format!(
                "keytool -genkey -v -keystore {} -alias {} -keyalg RSA -keysize 2048 -validity 10000",
                name, alias
            );

            tracing::info!("{}: {}", name.yellow(), "MISSING".red());
            tracing::info!("  Command: {}\n", cmd.green());

            // Create placeholder if not exists
            manage_placeholder(name)?;
        }
    }

    Ok(())
}

pub fn generate_keystore(path: &str, alias: &str, password: &str) -> Result<()> {
    use p12_keystore::{Certificate as P12Certificate, KeyStore, KeyStoreEntry, PrivateKeyChain};
    use rcgen::{CertificateParams, DnType, KeyPair};

    // 1. Generate a 2048-bit RSA key pair using openssl and load it into rcgen KeyPair
    let rsa = openssl::rsa::Rsa::generate(2048)
        .map_err(|e| anyhow::anyhow!("Failed to generate RSA key pair: {}", e))?;
    let pkey = openssl::pkey::PKey::from_rsa(rsa)
        .map_err(|e| anyhow::anyhow!("Failed to wrap RSA key in PKey: {}", e))?;
    let pem = pkey
        .private_key_to_pem_pkcs8()
        .map_err(|e| anyhow::anyhow!("Failed to serialize private key to PKCS#8 PEM: {}", e))?;
    let pem_str = std::str::from_utf8(&pem)
        .map_err(|e| anyhow::anyhow!("Failed to parse PEM bytes as UTF-8 string: {}", e))?;

    // 2. Import into rcgen KeyPair
    let key_pair = KeyPair::from_pem(pem_str)
        .map_err(|e| anyhow::anyhow!("Failed to import key pair into rcgen: {}", e))?;

    // 3. Generate a self-signed certificate using rcgen
    let mut params = CertificateParams::new(vec![alias.to_string()])
        .map_err(|e| anyhow::anyhow!("Failed to construct certificate parameters: {}", e))?;

    params.distinguished_name.push(DnType::CommonName, alias);

    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| anyhow::anyhow!("Failed to sign certificate: {}", e))?;

    // 4. Wrap them in a PKCS#12 envelope with the given alias and password
    let p12_cert = P12Certificate::from_der(cert.der().as_ref())
        .map_err(|e| anyhow::anyhow!("Failed to parse certificate for PKCS#12: {}", e))?;

    let private_key_der = key_pair.serialize_der();
    let local_key_id = alias.as_bytes();
    let private_key_chain = PrivateKeyChain::new(private_key_der, local_key_id, vec![p12_cert]);

    let mut keystore = KeyStore::new();
    keystore.add_entry(alias, KeyStoreEntry::PrivateKeyChain(private_key_chain));

    let p12_bytes = keystore
        .writer(password)
        .write()
        .map_err(|e| anyhow::anyhow!("Failed to write PKCS#12 keystore: {}", e))?;

    // 5. Write the resulting bytes to the target path
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, p12_bytes)?;

    Ok(())
}

pub fn manage_placeholder(keystore_path: &str) -> Result<()> {
    let placeholder_path = format!("{}.placeholder", keystore_path);
    if !Path::new(&placeholder_path).exists() {
        fs::write(&placeholder_path, "This is a placeholder for the actual keystore file. The real keystore should NOT be committed to version control.")?;
        tracing::info!(
            "{} Created placeholder '{}'.",
            "INFO".blue(),
            placeholder_path
        );
    }
    Ok(())
}

/// Check status of keystores
pub fn check_status() -> Result<()> {
    run_check_status()
}

fn run_check_status() -> Result<()> {
    let targets = vec![
        "barbarian-road-mashines-key.keystore",
        "zombie-key.keystore",
        "hang3d-nightmare-keystore.keystore",
    ];

    tracing::info!("\n{} Keystore Status:", "Crypto".bold());
    for name in targets {
        let exists = Path::new(name).exists();
        let placeholder_exists = Path::new(&format!("{}.placeholder", name)).exists();

        let status = if exists {
            "PRESENT".green()
        } else if placeholder_exists {
            "MISSING (Placeholder present)".yellow()
        } else {
            "MISSING".red()
        };

        tracing::info!("  - {}: {}", name, status);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use p12_keystore::KeyStore;
    use tempfile::tempdir;

    #[test]
    fn test_generate_keystore_native() {
        let dir = tempdir().unwrap();
        let keystore_path = dir.path().join("test.keystore");
        let path_str = keystore_path.to_str().unwrap();
        let alias = "test-alias";
        let password = "test-password";

        // Generate the keystore
        let gen_result = generate_keystore(path_str, alias, password);
        assert!(
            gen_result.is_ok(),
            "Keystore generation failed: {:?}",
            gen_result.err()
        );

        // Verify the keystore exists
        assert!(keystore_path.exists(), "Keystore file was not created");

        // Read and parse the keystore using p12-keystore
        let bytes = fs::read(&keystore_path).unwrap();
        let keystore = KeyStore::from_pkcs12(&bytes, password);
        assert!(
            keystore.is_ok(),
            "Failed to parse generated keystore: {:?}",
            keystore.err()
        );
        let keystore = keystore.unwrap();

        // Check entry exists with the specified alias
        let entry = keystore.entry(alias);
        assert!(
            entry.is_some(),
            "Keystore does not contain the entry for alias '{}'",
            alias
        );

        // Verify the key type is RSA and size is 2048-bit (256 bytes) using openssl
        let p12 =
            openssl::pkcs12::Pkcs12::from_der(&bytes).expect("Failed to parse PKCS#12 from bytes");
        let parsed = p12
            .parse2(password)
            .expect("Failed to decrypt/parse PKCS#12");
        let pkey = parsed.pkey.expect("No private key found in PKCS#12");
        let rsa = pkey.rsa().expect("Private key is not an RSA key");
        assert_eq!(rsa.size(), 256, "RSA key size is not 2048 bits");
    }

    #[test]
    fn manage_placeholder_creates_file_when_missing() {
        let dir = tempdir().unwrap();
        let ks = dir.path().join("test.keystore").to_str().unwrap().to_string();
        manage_placeholder(&ks).unwrap();
        let placeholder = format!("{}.placeholder", ks);
        assert!(Path::new(&placeholder).exists(), "placeholder not created");
        let content = fs::read_to_string(&placeholder).unwrap();
        assert!(content.contains("placeholder"), "placeholder file content wrong");
    }

    #[test]
    fn manage_placeholder_idempotent() {
        let dir = tempdir().unwrap();
        let ks = dir.path().join("k.keystore").to_str().unwrap().to_string();
        manage_placeholder(&ks).unwrap();
        manage_placeholder(&ks).unwrap(); // second call must not panic or error
        let count = dir.path().read_dir().unwrap().count();
        assert_eq!(count, 1, "second call must not create extra files");
    }

    #[test]
    fn check_status_does_not_panic() {
        // check_status() reads filenames from cwd — just assert it doesn't panic.
        let _ = check_status();
    }

    #[test]
    fn generate_keystore_nonexistent_dir_returns_error() {
        let result = generate_keystore("/nonexistent/dir/k.keystore", "alias", "pw");
        assert!(result.is_err());
    }
}
