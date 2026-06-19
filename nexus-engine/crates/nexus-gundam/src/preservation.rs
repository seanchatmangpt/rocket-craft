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
