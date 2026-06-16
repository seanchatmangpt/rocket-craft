use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KitBarcode {
    pub raw: String,
    pub kit_id: String,
    pub series_code: String,
    pub tier: KitTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KitTier {
    Hg,
    Rg,
    Mg,
    Pg,
}

impl PartialOrd for KitTier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for KitTier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl KitTier {
    pub fn digital_bonus_credits(&self) -> u32 {
        match self {
            KitTier::Hg => 300,
            KitTier::Rg => 600,
            KitTier::Mg => 900,
            KitTier::Pg => 1500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArUnlock {
    pub barcode: KitBarcode,
    pub digital_suit_id: String,
    pub bonus_credits: u32,
    pub exclusive_colorway: Option<String>,
    pub ar_nonce: String,
}

pub struct ArBridgeRegistry {
    redeemed_nonces: std::collections::HashSet<String>,
    kit_database: Vec<KitEntry>,
}

#[derive(Debug, Clone)]
struct KitEntry {
    kit_id: String,
    suit_id: String,
    tier: KitTier,
    series: String,
}

impl ArBridgeRegistry {
    pub fn new() -> Self {
        let mut registry = ArBridgeRegistry {
            redeemed_nonces: std::collections::HashSet::new(),
            kit_database: vec![],
        };
        registry.register_kit("HG-AERIAL-001", "XVX-016_Gundam-Aerial", KitTier::Hg, "WFM");
        registry.register_kit("RG-NU-001", "RX-93_Nu-Gundam", KitTier::Rg, "UC");
        registry.register_kit("MG-WING-ZERO-001", "XXXG-00W0_Wing-Zero", KitTier::Mg, "WING");
        registry.register_kit("PG-UNICORN-001", "RX-0_Unicorn-Gundam", KitTier::Pg, "UC");
        registry.register_kit("MG-FREEDOM-001", "ZGMF-X10A_Freedom-Gundam", KitTier::Mg, "SEED");
        registry.register_kit("PG-FREEDOM-001", "ZGMF-X10A_Freedom-Gundam", KitTier::Pg, "SEED");
        registry.register_kit("RG-ZETA-001", "MSZ-006_Zeta-Gundam", KitTier::Rg, "UC");
        registry.register_kit("MG-BARBATOS-001", "ASW-G-08_Barbatos-Lupus-Rex", KitTier::Mg, "IBO");
        registry
    }

    fn register_kit(&mut self, kit_id: &str, suit_id: &str, tier: KitTier, series: &str) {
        self.kit_database.push(KitEntry {
            kit_id: kit_id.to_string(),
            suit_id: suit_id.to_string(),
            tier,
            series: series.to_string(),
        });
    }

    pub fn redeem(&mut self, raw_barcode: &str, player_id: u64) -> Result<ArUnlock, ArError> {
        let nonce = self.generate_nonce(raw_barcode, player_id);

        if self.redeemed_nonces.contains(&nonce) {
            return Err(ArError::AlreadyRedeemed { barcode: raw_barcode.to_string() });
        }

        // Format: "GN-{tier}-{kit_id}"  e.g. "GN-HG-HG-AERIAL-001"
        let parts: Vec<&str> = raw_barcode.split('-').collect();
        if parts.len() < 3 || parts[0] != "GN" {
            return Err(ArError::InvalidBarcode(raw_barcode.to_string()));
        }

        let tier_code = parts[1];
        let kit_id = parts[2..].join("-");

        let tier = match tier_code {
            "HG" => KitTier::Hg,
            "RG" => KitTier::Rg,
            "MG" => KitTier::Mg,
            "PG" => KitTier::Pg,
            _ => return Err(ArError::UnknownTier(tier_code.to_string())),
        };

        let entry = self.kit_database.iter()
            .find(|e| e.kit_id == kit_id && e.tier == tier)
            .ok_or_else(|| ArError::KitNotFound(kit_id.clone()))?;

        let suit_id = entry.suit_id.clone();
        let series = entry.series.clone();

        self.redeemed_nonces.insert(nonce.clone());

        Ok(ArUnlock {
            barcode: KitBarcode {
                raw: raw_barcode.to_string(),
                kit_id: kit_id.clone(),
                series_code: series,
                tier,
            },
            digital_suit_id: suit_id,
            bonus_credits: tier.digital_bonus_credits(),
            exclusive_colorway: if tier >= KitTier::Mg {
                Some(format!("{}-GUNPLA-COLOR", kit_id))
            } else {
                None
            },
            ar_nonce: nonce,
        })
    }

    fn generate_nonce(&self, barcode: &str, player_id: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(barcode.as_bytes());
        hasher.update(&player_id.to_le_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn is_kit_registered(&self, kit_id: &str) -> bool {
        self.kit_database.iter().any(|e| e.kit_id == kit_id)
    }

    pub fn redeemed_count(&self) -> usize {
        self.redeemed_nonces.len()
    }
}

impl Default for ArBridgeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ArError {
    #[error("barcode already redeemed: {barcode}")]
    AlreadyRedeemed { barcode: String },
    #[error("invalid barcode format: {0}")]
    InvalidBarcode(String),
    #[error("unknown tier code: {0}")]
    UnknownTier(String),
    #[error("kit not found in registry: {0}")]
    KitNotFound(String),
}
