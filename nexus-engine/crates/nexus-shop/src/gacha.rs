use chrono::{DateTime, Utc};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GachaRarity {
    R,
    SR,
    SSR,
}

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
            // soft pity: +5% per pull over 70. At pull 70 extra=0; rate first exceeds base at pull 71.
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
        GachaEngine {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn from_server_entropy(player_id: u64, nonce: u64) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(player_id.to_le_bytes());
        hasher.update(nonce.to_le_bytes());
        let result = hasher.finalize();
        let seed = u64::from_le_bytes(result[..8].try_into().unwrap());
        GachaEngine {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn single_pull(
        &mut self,
        banner: &Banner,
        session: &mut PullSession,
    ) -> Result<PullResult, GachaError> {
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
            Ok(PullResult {
                item,
                pity_pull,
                pulls_used: 1,
                new_pity_count: new_count,
            })
        } else {
            let count = session.pulls_since_last_ssr;
            Ok(PullResult {
                item,
                pity_pull: false,
                pulls_used: 1,
                new_pity_count: count,
            })
        }
    }

    pub fn ten_pull(
        &mut self,
        banner: &Banner,
        session: &mut PullSession,
    ) -> Result<Vec<PullResult>, GachaError> {
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
                let sr_items: Vec<_> = banner
                    .items
                    .iter()
                    .filter(|i| i.rarity == GachaRarity::SR)
                    .collect();
                if !sr_items.is_empty() {
                    let idx = self.rng.random_range(0..sr_items.len());
                    last.item = sr_items[idx].clone();
                }
            }
        }

        Ok(results)
    }

    fn select_item(
        &mut self,
        banner: &Banner,
        rarity: GachaRarity,
        session: &PullSession,
    ) -> Result<GachaItem, GachaError> {
        let pool: Vec<_> = banner.items.iter().filter(|i| i.rarity == rarity).collect();
        if pool.is_empty() {
            let lower = if rarity == GachaRarity::SSR {
                GachaRarity::SR
            } else {
                GachaRarity::R
            };
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ssr_item(id: &str, rate_up: bool) -> GachaItem {
        GachaItem {
            id: id.into(),
            name: id.into(),
            rarity: GachaRarity::SSR,
            banner_id: "b".into(),
            is_rate_up: rate_up,
        }
    }
    fn sr_item(id: &str) -> GachaItem {
        GachaItem {
            id: id.into(),
            name: id.into(),
            rarity: GachaRarity::SR,
            banner_id: "b".into(),
            is_rate_up: false,
        }
    }
    fn r_item(id: &str) -> GachaItem {
        GachaItem {
            id: id.into(),
            name: id.into(),
            rarity: GachaRarity::R,
            banner_id: "b".into(),
            is_rate_up: false,
        }
    }

    fn banner_with(ssr: Vec<GachaItem>, sr: Vec<GachaItem>, r: Vec<GachaItem>) -> Banner {
        let mut items = ssr;
        items.extend(sr);
        items.extend(r);
        Banner::standard("b", "Test Banner", items)
    }

    fn session() -> PullSession {
        PullSession::new(1, "b".into())
    }

    // ── GachaRarity ordering ──────────────────────────────────────────────────

    #[test]
    fn rarity_ordering_r_lt_sr_lt_ssr() {
        assert!(GachaRarity::R < GachaRarity::SR);
        assert!(GachaRarity::SR < GachaRarity::SSR);
        assert!(GachaRarity::R < GachaRarity::SSR);
    }

    // ── PullSession::current_ssr_rate ─────────────────────────────────────────

    #[test]
    fn base_rate_before_soft_pity() {
        let mut s = session();
        s.pulls_since_last_ssr = 0;
        assert!((s.current_ssr_rate() - 0.03).abs() < 1e-9);
        s.pulls_since_last_ssr = 69;
        assert!((s.current_ssr_rate() - 0.03).abs() < 1e-9);
    }

    #[test]
    fn soft_pity_increases_rate_linearly_from_pull_70() {
        // pull 70: extra = (70-70)*0.05 = 0 → still base rate
        // pull 71: extra = 1*0.05 = 0.05 → first actual increase
        let mut s = session();
        s.pulls_since_last_ssr = 71;
        let r71 = s.current_ssr_rate();
        assert!(r71 > 0.03, "rate must exceed base at pull 71");
        s.pulls_since_last_ssr = 72;
        let r72 = s.current_ssr_rate();
        assert!(r72 > r71, "rate must keep increasing in soft pity window");
    }

    #[test]
    fn hard_pity_at_90_gives_guaranteed_ssr_rate() {
        let mut s = session();
        s.pulls_since_last_ssr = 90;
        assert_eq!(s.current_ssr_rate(), 1.0);
    }

    #[test]
    fn rate_caps_at_1_during_soft_pity() {
        let mut s = session();
        s.pulls_since_last_ssr = 88; // soft pity, well above floor
        assert!(s.current_ssr_rate() <= 1.0);
    }

    // ── GachaEngine::single_pull ──────────────────────────────────────────────

    #[test]
    fn pull_from_empty_banner_returns_error() {
        let banner = Banner::standard("b", "empty", vec![]);
        let mut engine = GachaEngine::new(42);
        let mut s = session();
        assert!(matches!(
            engine.single_pull(&banner, &mut s),
            Err(GachaError::EmptyBanner(_))
        ));
    }

    #[test]
    fn pull_advances_total_pulls_counter() {
        let banner = banner_with(vec![ssr_item("Wing", false)], vec![], vec![r_item("gm")]);
        let mut engine = GachaEngine::new(42);
        let mut s = session();
        engine.single_pull(&banner, &mut s).unwrap();
        assert_eq!(s.total_pulls, 1);
        engine.single_pull(&banner, &mut s).unwrap();
        assert_eq!(s.total_pulls, 2);
    }

    #[test]
    fn ssr_pull_resets_pity_counter() {
        // Force an SSR by setting pity to 90 before the pull.
        let banner = banner_with(vec![ssr_item("RX-78", false)], vec![], vec![r_item("gm")]);
        let mut engine = GachaEngine::new(0);
        let mut s = session();
        s.pulls_since_last_ssr = 89; // next pull = 90 → guaranteed SSR
        let result = engine.single_pull(&banner, &mut s).unwrap();
        assert_eq!(result.item.rarity, GachaRarity::SSR);
        assert!(result.pity_pull);
        assert_eq!(
            s.pulls_since_last_ssr, 0,
            "pity counter must reset after SSR"
        );
    }

    #[test]
    fn ssr_increments_ssr_count() {
        let banner = banner_with(vec![ssr_item("RX-78", false)], vec![], vec![r_item("gm")]);
        let mut engine = GachaEngine::new(0);
        let mut s = session();
        s.pulls_since_last_ssr = 89; // force SSR
        engine.single_pull(&banner, &mut s).unwrap();
        assert_eq!(s.ssr_count, 1);
    }

    // ── GachaEngine::ten_pull ─────────────────────────────────────────────────

    #[test]
    fn ten_pull_always_returns_exactly_10_results() {
        let banner = banner_with(
            vec![ssr_item("RX-78", false)],
            vec![sr_item("Zaku II")],
            vec![r_item("gm"), r_item("ball")],
        );
        let mut engine = GachaEngine::new(999);
        let mut s = session();
        let results = engine.ten_pull(&banner, &mut s).unwrap();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn ten_pull_advances_total_pulls_by_10() {
        let banner = banner_with(
            vec![ssr_item("Wing", false)],
            vec![sr_item("Zaku II")],
            vec![r_item("gm")],
        );
        let mut engine = GachaEngine::new(777);
        let mut s = session();
        engine.ten_pull(&banner, &mut s).unwrap();
        assert_eq!(s.total_pulls, 10);
    }

    // ── GachaRarity ordering edge cases ───────────────────────────────────────

    #[test]
    fn same_rarity_is_equal() {
        assert_eq!(GachaRarity::SSR, GachaRarity::SSR);
        assert_eq!(GachaRarity::SR, GachaRarity::SR);
        assert_eq!(GachaRarity::R, GachaRarity::R);
    }
}
