use std::collections::{HashMap, HashSet};
use unify_receipts::receipt::Receipt;

/// An LSP capability that a server can provide.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Capability {
    TextDocumentSync,
    Completion,
    Hover,
    Definition,
    References,
    DocumentHighlight,
    DocumentSymbol,
    WorkspaceSymbol,
    CodeAction,
    CodeLens,
    DocumentFormatting,
    Rename,
    Diagnostics,
    InlayHints,
    Custom(String),
}

impl Capability {
    fn key(&self) -> String {
        match self {
            Capability::TextDocumentSync => "TextDocumentSync".to_owned(),
            Capability::Completion => "Completion".to_owned(),
            Capability::Hover => "Hover".to_owned(),
            Capability::Definition => "Definition".to_owned(),
            Capability::References => "References".to_owned(),
            Capability::DocumentHighlight => "DocumentHighlight".to_owned(),
            Capability::DocumentSymbol => "DocumentSymbol".to_owned(),
            Capability::WorkspaceSymbol => "WorkspaceSymbol".to_owned(),
            Capability::CodeAction => "CodeAction".to_owned(),
            Capability::CodeLens => "CodeLens".to_owned(),
            Capability::DocumentFormatting => "DocumentFormatting".to_owned(),
            Capability::Rename => "Rename".to_owned(),
            Capability::Diagnostics => "Diagnostics".to_owned(),
            Capability::InlayHints => "InlayHints".to_owned(),
            Capability::Custom(s) => format!("Custom:{}", s),
        }
    }
}

/// A set of LSP capabilities backed by BLAKE3 receipts.
pub struct CapabilitySet {
    capabilities: HashSet<Capability>,
    receipts: HashMap<String, Receipt>,
}

impl CapabilitySet {
    /// Create an empty capability set.
    pub fn empty() -> Self {
        Self {
            capabilities: HashSet::new(),
            receipts: HashMap::new(),
        }
    }

    /// Grant a capability, creating a BLAKE3 receipt over `data`.
    /// Returns a reference to the stored receipt.
    pub fn grant(&mut self, cap: Capability, data: &[u8]) -> &Receipt {
        let key = cap.key();
        let receipt = Receipt::new(&key, data);
        self.capabilities.insert(cap);
        self.receipts.insert(key.clone(), receipt);
        self.receipts.get(&key).expect("just inserted")
    }

    /// Revoke a capability, removing its receipt.
    pub fn revoke(&mut self, cap: &Capability) {
        let key = cap.key();
        self.capabilities.remove(cap);
        self.receipts.remove(&key);
    }

    /// Returns `true` if this set contains `cap`.
    pub fn has(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Returns the receipt for `cap`, if present.
    pub fn receipt_for(&self, cap: &Capability) -> Option<&Receipt> {
        self.receipts.get(&cap.key())
    }

    /// Verify all stored receipts.
    ///
    /// Because we only store the hash (not the original data), verification
    /// checks that every granted capability has a receipt whose key matches
    /// a capability in the set.
    pub fn verify_all(&self) -> bool {
        // Every capability must have a receipt; every receipt must have a capability.
        for cap in &self.capabilities {
            if !self.receipts.contains_key(&cap.key()) {
                return false;
            }
        }
        for key in self.receipts.keys() {
            // Find any capability whose key matches this receipt.
            let found = self.capabilities.iter().any(|c| &c.key() == key);
            if !found {
                return false;
            }
        }
        true
    }

    /// Number of granted capabilities.
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Returns `true` if no capabilities are granted.
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CapabilitySet::empty ───────────────────────────────────────────────────

    #[test]
    fn empty_has_no_capabilities() {
        let set = CapabilitySet::empty();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    // ── grant ─────────────────────────────────────────────────────────────────

    #[test]
    fn grant_adds_capability_to_set() {
        let mut set = CapabilitySet::empty();
        set.grant(Capability::Hover, b"data");
        assert!(set.has(&Capability::Hover));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn grant_creates_receipt_with_matching_key() {
        let mut set = CapabilitySet::empty();
        let receipt = set.grant(Capability::Completion, b"test");
        assert_eq!(receipt.key, "Completion");
        assert_eq!(receipt.hash.len(), 64); // BLAKE3 hex
    }

    #[test]
    fn grant_same_capability_twice_keeps_one() {
        let mut set = CapabilitySet::empty();
        set.grant(Capability::Hover, b"v1");
        set.grant(Capability::Hover, b"v2");
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn grant_custom_capability_is_retrievable() {
        let mut set = CapabilitySet::empty();
        set.grant(Capability::Custom("myExt".into()), b"ext");
        assert!(set.has(&Capability::Custom("myExt".into())));
    }

    // ── revoke ────────────────────────────────────────────────────────────────

    #[test]
    fn revoke_removes_capability_and_receipt() {
        let mut set = CapabilitySet::empty();
        set.grant(Capability::Diagnostics, b"d");
        set.revoke(&Capability::Diagnostics);
        assert!(!set.has(&Capability::Diagnostics));
        assert!(set.receipt_for(&Capability::Diagnostics).is_none());
        assert!(set.is_empty());
    }

    #[test]
    fn revoke_nonexistent_is_a_noop() {
        let mut set = CapabilitySet::empty();
        set.revoke(&Capability::Hover); // should not panic
        assert!(set.is_empty());
    }

    // ── receipt_for ───────────────────────────────────────────────────────────

    #[test]
    fn receipt_for_returns_none_before_grant() {
        let set = CapabilitySet::empty();
        assert!(set.receipt_for(&Capability::Definition).is_none());
    }

    #[test]
    fn receipt_for_returns_some_after_grant() {
        let mut set = CapabilitySet::empty();
        set.grant(Capability::References, b"refs");
        assert!(set.receipt_for(&Capability::References).is_some());
    }

    // ── verify_all ────────────────────────────────────────────────────────────

    #[test]
    fn verify_all_passes_on_empty_set() {
        assert!(CapabilitySet::empty().verify_all());
    }

    #[test]
    fn verify_all_passes_after_valid_grants() {
        let mut set = CapabilitySet::empty();
        set.grant(Capability::Hover, b"h");
        set.grant(Capability::Completion, b"c");
        assert!(set.verify_all());
    }
}
