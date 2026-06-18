use rand::{SeedableRng, RngExt};
use rand_chacha::ChaCha8Rng;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GachaRarity { R, SR, SSR }

impl PartialOrd for GachaRarity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GachaRarity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use GachaRarity::*;
        match (self, other) {
            (R, R) | (SR, SR) | (SSR, SSR) => std::cmp::Ordering::Equal,
            (R, _) => std::cmp::Ordering::Less,
            (_, R) => std::cmp::Ordering::Greater,
            (SR, SSR) => std::cmp::Ordering::Less,
            (SSR, SR) => std::cmp::Ordering::Greater,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GachaItem {
    pub id: String,
    pub name: String,
    pub rarity: GachaRarity,
    pub banner_id: String,
    pub is_rate_up: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BannerType {
    Limited,
    Standard,
    Collab,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Banner {
    pub id: String,
    pub name: String,
    pub banner_type: BannerType,
    pub items: Vec<GachaItem>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub ssr_rate: f64,
    pub sr_rate: f64,
    pub r_rate: f64,
    pub rate_up_share: f64,
}

impl Banner {
    pub fn standard(id: &str, name: &str, items: Vec<GachaItem>) -> Self {
        Banner {
            id: id.to_string(),
            name: name.to_string(),
            banner_type: BannerType::Standard,
            items,
            starts_at: Utc::now(),
            ends_at: Utc::now() + chrono::Duration::days(36_500),
            ssr_rate: 0.03,
            sr_rate: 0.12,
            r_rate: 0.85,
            rate_up_share: 0.5,
        }
    }

    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        now >= self.starts_at && now <= self.ends_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullSession {
    pub player_id: u64,
    pub banner_id: String,
    pub pulls_since_last_ssr: u32,
    pub total_pulls: u64,
    pub ssr_count: u32,
    pub rate_up_pity_active: bool,
}

impl PullSession {
    pub fn new(player_id: u64, banner_id: String) -> Self {
        PullSession {
            player_id,
            banner_id,
            pulls_since_last_ssr: 0,
            total_pulls: 0,
            ssr_count: 0,
            rate_up_pity_active: false,
        }
    }

    pub fn current_ssr_rate(&self) -> f64 {
        let base = 0.03;
        if self.pulls_since_last_ssr < 70 {
            base
        } else if self.pulls_since_last_ssr < 90 {
            let extra = (self.pulls_since_last_ssr - 70) as f64 * 0.05;
            (base + extra).min(1.0)
        } else {
            1.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResult {
    pub item: GachaItem,
    pub pity_pull: bool,
    pub pulls_used: u32,
    pub new_pity_count: u32,
}

pub struct GachaEngine {
    rng: ChaCha8Rng,
}

impl GachaEngine {
    pub fn new(seed: u64) -> Self {
        GachaEngine { rng: ChaCha8Rng::seed_from_u64(seed) }
    }

    pub fn from_server_entropy(player_id: u64, nonce: u64) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(player_id.to_le_bytes());
        hasher.update(nonce.to_le_bytes());
        let result = hasher.finalize();
        let seed = u64::from_le_bytes(result[..8].try_into().unwrap());
        GachaEngine { rng: ChaCha8Rng::seed_from_u64(seed) }
    }

    pub fn single_pull(&mut self, banner: &Banner, session: &mut PullSession) -> Result<PullResult, GachaError> {
        if !banner.is_active() {
            return Err(GachaError::BannerExpired(banner.id.clone()));
        }
        if banner.items.is_empty() {
            return Err(GachaError::EmptyBanner(banner.id.clone()));
        }

        session.pulls_since_last_ssr += 1;
        session.total_pulls += 1;

        let roll: f64 = self.rng.random();
        let ssr_rate = session.current_ssr_rate();
        let pity_pull = session.pulls_since_last_ssr >= 90;

        let rarity = if roll < ssr_rate || pity_pull {
            GachaRarity::SSR
        } else if roll < ssr_rate + banner.sr_rate {
            GachaRarity::SR
        } else {
            GachaRarity::R
        };

        let item = self.select_item(banner, rarity, session)?;

        if item.rarity == GachaRarity::SSR {
            let new_count = session.pulls_since_last_ssr;
            session.pulls_since_last_ssr = 0;
            session.ssr_count += 1;
            Ok(PullResult { item, pity_pull, pulls_used: 1, new_pity_count: new_count })
        } else {
            let count = session.pulls_since_last_ssr;
            Ok(PullResult { item, pity_pull: false, pulls_used: 1, new_pity_count: count })
        }
    }

    pub fn ten_pull(&mut self, banner: &Banner, session: &mut PullSession) -> Result<Vec<PullResult>, GachaError> {
        let mut results = Vec::with_capacity(10);
        let mut has_sr_or_above = false;

        for _i in 0..10 {
            let result = self.single_pull(banner, session)?;
            if result.item.rarity >= GachaRarity::SR {
                has_sr_or_above = true;
            }
            results.push(result);
        }

        if !has_sr_or_above {
            if let Some(last) = results.last_mut() {
                let sr_items: Vec<_> = banner.items.iter().filter(|i| i.rarity == GachaRarity::SR).collect();
                if !sr_items.is_empty() {
                    let idx = self.rng.random_range(0..sr_items.len());
                    last.item = sr_items[idx].clone();
                }
            }
        }

        Ok(results)
    }

    fn select_item(&mut self, banner: &Banner, rarity: GachaRarity, session: &PullSession) -> Result<GachaItem, GachaError> {
        let pool: Vec<_> = banner.items.iter().filter(|i| i.rarity == rarity).collect();
        if pool.is_empty() {
            let lower = if rarity == GachaRarity::SSR { GachaRarity::SR } else { GachaRarity::R };
            let lower_pool: Vec<_> = banner.items.iter().filter(|i| i.rarity == lower).collect();
            if lower_pool.is_empty() {
                return Err(GachaError::NoItemsInPool {
                    banner_id: banner.id.clone(),
                    rarity: format!("{:?}", rarity),
                });
            }
            let idx = self.rng.random_range(0..lower_pool.len());
            return Ok(lower_pool[idx].clone());
        }

        if rarity == GachaRarity::SSR {
            let rate_up_items: Vec<_> = pool.iter().filter(|i| i.is_rate_up).collect();
            if !rate_up_items.is_empty() {
                let roll: f64 = self.rng.random();
                if roll < banner.rate_up_share || session.rate_up_pity_active {
                    let idx = self.rng.random_range(0..rate_up_items.len());
                    return Ok((*rate_up_items[idx]).clone());
                }
            }
        }

        let idx = self.rng.random_range(0..pool.len());
        Ok(pool[idx].clone())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GachaError {
    #[error("banner {0} has expired")]
    BannerExpired(String),
    #[error("banner {0} has no items")]
    EmptyBanner(String),
    #[error("no items of rarity {rarity} in banner {banner_id}")]
    NoItemsInPool { banner_id: String, rarity: String },
    #[error("insufficient credits: need {need}, have {have}")]
    InsufficientCredits { need: u32, have: u32 },
}
