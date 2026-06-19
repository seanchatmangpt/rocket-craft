use crate::generated_gundam::{OntologyName, PreservationDomainCategory};
use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct PreservationArtifact<Dom: PreservationDomainCategory> {
    pub id: String,
    pub domain: Dom, // Enforce a typed domain category
    pub name: String,
    pub original_url: Option<String>,
    pub metadata: HashMap<String, String>,
    pub assets_blob: Vec<u8>,
    pub hash: String, // SHA-256 hash of assets_blob
}

pub trait PreservationLayer {
    fn preserve_artifact<Dom: PreservationDomainCategory + OntologyName>(
        &self,
        artifact: PreservationArtifact<Dom>,
    ) -> Result<String>;
    fn retrieve_artifact<Dom: PreservationDomainCategory + OntologyName + Default>(
        &self,
        id: &str,
    ) -> Result<PreservationArtifact<Dom>>;
    fn list_artifacts_by_domain<Dom: PreservationDomainCategory + OntologyName + Default>(
        &self,
        domain: Dom,
    ) -> Result<Vec<PreservationArtifact<Dom>>>;
    fn verify_integrity(&self, id: &str) -> Result<bool>;
}

// Internal non-generic structure for storage
#[derive(Debug, Clone)]
struct StoredPreservationArtifact {
    id: String,
    domain: String,
    name: String,
    original_url: Option<String>,
    metadata: HashMap<String, String>,
    assets_blob: Vec<u8>,
    hash: String,
}

#[derive(Default)]
pub struct GundamPreservationManager {
    registry: RwLock<HashMap<String, StoredPreservationArtifact>>,
}

impl GundamPreservationManager {
    pub fn new() -> Self {
        Self {
            registry: RwLock::new(HashMap::new()),
        }
    }
}

impl PreservationLayer for GundamPreservationManager {
    fn preserve_artifact<Dom: PreservationDomainCategory + OntologyName>(
        &self,
        mut artifact: PreservationArtifact<Dom>,
    ) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(&artifact.assets_blob);
        let calculated_hash = format!("{:x}", hasher.finalize());

        // Ensure the hash in the artifact matches or update it
        artifact.hash = calculated_hash.clone();

        let stored = StoredPreservationArtifact {
            id: artifact.id,
            domain: Dom::ontology_name().to_string(),
            name: artifact.name,
            original_url: artifact.original_url,
            metadata: artifact.metadata,
            assets_blob: artifact.assets_blob,
            hash: calculated_hash.clone(),
        };

        let mut write_guard = self
            .registry
            .write()
            .map_err(|e| anyhow!("Failed to acquire write lock: {}", e))?;
        write_guard.insert(stored.id.clone(), stored);

        Ok(calculated_hash)
    }

    fn retrieve_artifact<Dom: PreservationDomainCategory + OntologyName + Default>(
        &self,
        id: &str,
    ) -> Result<PreservationArtifact<Dom>> {
        let read_guard = self
            .registry
            .read()
            .map_err(|e| anyhow!("Failed to acquire read lock: {}", e))?;
        let stored = read_guard
            .get(id)
            .ok_or_else(|| anyhow!("Artifact not found: {}", id))?;

        let expected_domain = Dom::ontology_name();
        if stored.domain != expected_domain {
            return Err(anyhow!(
                "Domain mismatch: expected {}, found {}",
                expected_domain,
                stored.domain
            ));
        }

        Ok(PreservationArtifact {
            id: stored.id.clone(),
            domain: Dom::default(),
            name: stored.name.clone(),
            original_url: stored.original_url.clone(),
            metadata: stored.metadata.clone(),
            assets_blob: stored.assets_blob.clone(),
            hash: stored.hash.clone(),
        })
    }

    fn list_artifacts_by_domain<Dom: PreservationDomainCategory + OntologyName + Default>(
        &self,
        _domain: Dom,
    ) -> Result<Vec<PreservationArtifact<Dom>>> {
        let read_guard = self
            .registry
            .read()
            .map_err(|e| anyhow!("Failed to acquire read lock: {}", e))?;
        let expected_domain = Dom::ontology_name();
        let filtered = read_guard
            .values()
            .filter(|art| art.domain == expected_domain)
            .map(|stored| PreservationArtifact {
                id: stored.id.clone(),
                domain: Dom::default(),
                name: stored.name.clone(),
                original_url: stored.original_url.clone(),
                metadata: stored.metadata.clone(),
                assets_blob: stored.assets_blob.clone(),
                hash: stored.hash.clone(),
            })
            .collect();
        Ok(filtered)
    }

    fn verify_integrity(&self, id: &str) -> Result<bool> {
        let read_guard = self
            .registry
            .read()
            .map_err(|e| anyhow!("Failed to acquire read lock: {}", e))?;
        let stored = read_guard
            .get(id)
            .ok_or_else(|| anyhow!("Artifact not found: {}", id))?;

        let mut hasher = Sha256::new();
        hasher.update(&stored.assets_blob);
        let calculated_hash = format!("{:x}", hasher.finalize());

        Ok(calculated_hash == stored.hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated_gundam::{Arcades, FlashGames};

    fn mgr() -> GundamPreservationManager {
        GundamPreservationManager::new()
    }

    fn arcades_artifact(id: &str, data: &[u8]) -> PreservationArtifact<Arcades> {
        PreservationArtifact {
            id: id.to_string(),
            domain: Arcades,
            name: format!("artifact-{id}"),
            original_url: None,
            metadata: HashMap::new(),
            assets_blob: data.to_vec(),
            hash: String::new(), // computed by preserve_artifact
        }
    }

    // ── preserve + retrieve round-trip ────────────────────────────────────────

    #[test]
    fn preserve_returns_sha256_hex_string() {
        let m = mgr();
        let hash = m.preserve_artifact(arcades_artifact("a1", b"hello")).unwrap();
        assert_eq!(hash.len(), 64); // SHA-256 is 32 bytes = 64 hex chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn retrieve_after_preserve_recovers_data() {
        let m = mgr();
        m.preserve_artifact(arcades_artifact("a2", b"gundam data")).unwrap();
        let art: PreservationArtifact<Arcades> = m.retrieve_artifact("a2").unwrap();
        assert_eq!(art.assets_blob, b"gundam data");
        assert_eq!(art.name, "artifact-a2");
    }

    #[test]
    fn retrieve_nonexistent_returns_error() {
        let m = mgr();
        let result: Result<PreservationArtifact<Arcades>> = m.retrieve_artifact("missing");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn retrieve_with_wrong_domain_returns_domain_mismatch_error() {
        let m = mgr();
        m.preserve_artifact(arcades_artifact("a3", b"data")).unwrap();
        // retrieve as FlashGames when stored as Arcades
        let result: Result<PreservationArtifact<FlashGames>> = m.retrieve_artifact("a3");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mismatch"));
    }

    // ── verify_integrity ─────────────────────────────────────────────────────

    #[test]
    fn verify_integrity_passes_for_stored_artifact() {
        let m = mgr();
        m.preserve_artifact(arcades_artifact("a4", b"blob")).unwrap();
        assert!(m.verify_integrity("a4").unwrap());
    }

    #[test]
    fn verify_integrity_missing_id_returns_error() {
        let m = mgr();
        assert!(m.verify_integrity("no-such-id").is_err());
    }

    // ── list_artifacts_by_domain ──────────────────────────────────────────────

    #[test]
    fn list_by_domain_returns_only_matching_domain() {
        let m = mgr();
        m.preserve_artifact(arcades_artifact("arc1", b"d1")).unwrap();
        m.preserve_artifact(arcades_artifact("arc2", b"d2")).unwrap();
        // Store a FlashGames artifact too
        m.preserve_artifact(PreservationArtifact {
            id: "flash1".into(), domain: FlashGames, name: "flash-game".into(),
            original_url: None, metadata: HashMap::new(),
            assets_blob: b"flash data".to_vec(), hash: String::new(),
        }).unwrap();

        let arcades_list: Vec<PreservationArtifact<Arcades>> =
            m.list_artifacts_by_domain(Arcades).unwrap();
        assert_eq!(arcades_list.len(), 2);
        assert!(arcades_list.iter().all(|a| a.domain == Arcades));
    }

    #[test]
    fn list_by_domain_empty_when_none_stored() {
        let m = mgr();
        let list: Vec<PreservationArtifact<Arcades>> = m.list_artifacts_by_domain(Arcades).unwrap();
        assert!(list.is_empty());
    }

    // ── hash determinism ──────────────────────────────────────────────────────

    #[test]
    fn same_data_produces_same_hash() {
        let m = mgr();
        let h1 = m.preserve_artifact(arcades_artifact("x1", b"repeatable")).unwrap();
        let h2 = m.preserve_artifact(arcades_artifact("x2", b"repeatable")).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_data_produces_different_hash() {
        let m = mgr();
        let h1 = m.preserve_artifact(arcades_artifact("y1", b"aaa")).unwrap();
        let h2 = m.preserve_artifact(arcades_artifact("y2", b"bbb")).unwrap();
        assert_ne!(h1, h2);
    }

    // ── metadata round-trip ────────────────────────────────────────────────────

    #[test]
    fn metadata_preserved_through_store_and_retrieve() {
        let m = mgr();
        let mut art = arcades_artifact("m1", b"meta test");
        art.metadata.insert("platform".into(), "Atari 2600".into());
        art.metadata.insert("year".into(), "1977".into());
        m.preserve_artifact(art).unwrap();
        let retrieved: PreservationArtifact<Arcades> = m.retrieve_artifact("m1").unwrap();
        assert_eq!(retrieved.metadata.get("platform").unwrap(), "Atari 2600");
        assert_eq!(retrieved.metadata.get("year").unwrap(), "1977");
    }
}
