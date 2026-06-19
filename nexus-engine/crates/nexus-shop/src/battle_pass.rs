use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlePassSeason {
    pub season_id: u32,
    pub name: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub free_rewards: Vec<PassReward>,
    pub premium_rewards: Vec<PassReward>,
    pub cost_credits: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassReward {
    pub tier: u32,
    pub reward_type: RewardType,
    pub name: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewardType {
    Cosmetic,
    Credits,
    SuitSkin,
    PilotSuit,
    Emblem,
    Experience,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPassState {
    pub player_id: u64,
    pub season_id: u32,
    pub is_premium: bool,
    pub current_tier: u32,
    pub pass_xp: u64,
    pub claimed_tiers: Vec<u32>,
    pub activated_at: Option<DateTime<Utc>>,
}

impl PlayerPassState {
    pub fn new(player_id: u64, season_id: u32) -> Self {
        PlayerPassState {
            player_id,
            season_id,
            is_premium: false,
            current_tier: 0,
            pass_xp: 0,
            claimed_tiers: vec![],
            activated_at: None,
        }
    }

    pub fn activate_premium(&mut self) -> Result<(), BattlePassError> {
        if self.is_premium {
            return Err(BattlePassError::AlreadyPremium);
        }
        self.is_premium = true;
        self.activated_at = Some(Utc::now());
        Ok(())
    }

    pub fn earn_xp(&mut self, xp: u64) -> Vec<u32> {
        self.pass_xp += xp;
        let new_tier = (self.pass_xp / 1000) as u32;
        let prev_tier = self.current_tier;
        self.current_tier = new_tier.min(40);
        (prev_tier + 1..=self.current_tier).collect()
    }

    pub fn claim_reward(
        &mut self,
        tier: u32,
        season: &BattlePassSeason,
    ) -> Result<PassReward, BattlePassError> {
        if tier > self.current_tier {
            return Err(BattlePassError::TierNotReached {
                tier,
                current: self.current_tier,
            });
        }
        if self.claimed_tiers.contains(&tier) {
            return Err(BattlePassError::AlreadyClaimed(tier));
        }

        let rewards = if tier > 10 || self.is_premium {
            &season.premium_rewards
        } else {
            &season.free_rewards
        };

        let reward = rewards
            .iter()
            .find(|r| r.tier == tier)
            .ok_or(BattlePassError::RewardNotFound(tier))?
            .clone();

        self.claimed_tiers.push(tier);
        Ok(reward)
    }

    pub fn unclaimed_count(&self, _season: &BattlePassSeason) -> usize {
        let available: Vec<u32> = (1..=self.current_tier).collect();
        available
            .iter()
            .filter(|t| !self.claimed_tiers.contains(t))
            .count()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BattlePassError {
    #[error("already premium")]
    AlreadyPremium,
    #[error("tier {tier} not reached (current: {current})")]
    TierNotReached { tier: u32, current: u32 },
    #[error("tier {0} already claimed")]
    AlreadyClaimed(u32),
    #[error("reward for tier {0} not found in season")]
    RewardNotFound(u32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn season() -> BattlePassSeason {
        let free_rewards = (1u32..=20)
            .map(|t| PassReward {
                tier: t,
                reward_type: RewardType::Credits,
                name: format!("Free T{t}"),
                quantity: t * 100,
            })
            .collect();
        let premium_rewards = (1u32..=40)
            .map(|t| PassReward {
                tier: t,
                reward_type: if t > 10 { RewardType::SuitSkin } else { RewardType::Credits },
                name: format!("Premium T{t}"),
                quantity: t * 200,
            })
            .collect();
        BattlePassSeason {
            season_id: 1,
            name: "Season 1".into(),
            starts_at: Utc::now(),
            ends_at: Utc::now(),
            free_rewards,
            premium_rewards,
            cost_credits: 1000,
        }
    }

    fn player() -> PlayerPassState {
        PlayerPassState::new(1, 1)
    }

    // ── new / defaults ────────────────────────────────────────────────────────

    #[test]
    fn new_player_starts_free_at_tier_0_with_no_xp() {
        let p = player();
        assert!(!p.is_premium);
        assert_eq!(p.current_tier, 0);
        assert_eq!(p.pass_xp, 0);
        assert!(p.claimed_tiers.is_empty());
    }

    // ── activate_premium ──────────────────────────────────────────────────────

    #[test]
    fn activate_premium_sets_flag() {
        let mut p = player();
        p.activate_premium().unwrap();
        assert!(p.is_premium);
    }

    #[test]
    fn activate_premium_twice_returns_error() {
        let mut p = player();
        p.activate_premium().unwrap();
        let result = p.activate_premium();
        assert!(matches!(result, Err(BattlePassError::AlreadyPremium)));
    }

    // ── earn_xp / tier progression ───────────────────────────────────────────

    #[test]
    fn earn_xp_below_1000_does_not_advance_tier() {
        let mut p = player();
        let unlocked = p.earn_xp(999);
        assert!(unlocked.is_empty());
        assert_eq!(p.current_tier, 0);
    }

    #[test]
    fn earn_1000_xp_unlocks_tier_1() {
        let mut p = player();
        let unlocked = p.earn_xp(1000);
        assert_eq!(unlocked, vec![1]);
        assert_eq!(p.current_tier, 1);
    }

    #[test]
    fn earn_3000_xp_unlocks_tiers_1_to_3() {
        let mut p = player();
        let unlocked = p.earn_xp(3000);
        assert_eq!(unlocked, vec![1, 2, 3]);
        assert_eq!(p.current_tier, 3);
    }

    #[test]
    fn earn_xp_caps_at_tier_40() {
        let mut p = player();
        p.earn_xp(999_999);
        assert_eq!(p.current_tier, 40);
    }

    // ── claim_reward ──────────────────────────────────────────────────────────

    #[test]
    fn free_player_can_claim_tier_1_free_reward() {
        let mut p = player();
        p.earn_xp(1000);
        let s = season();
        let reward = p.claim_reward(1, &s).unwrap();
        assert_eq!(reward.tier, 1);
        assert!(p.claimed_tiers.contains(&1));
    }

    #[test]
    fn claiming_ahead_of_current_tier_returns_error() {
        let mut p = player();
        let s = season();
        let result = p.claim_reward(5, &s);
        assert!(matches!(result, Err(BattlePassError::TierNotReached { tier: 5, current: 0 })));
    }

    #[test]
    fn claiming_same_tier_twice_returns_error() {
        let mut p = player();
        p.earn_xp(1000);
        let s = season();
        p.claim_reward(1, &s).unwrap();
        let result = p.claim_reward(1, &s);
        assert!(matches!(result, Err(BattlePassError::AlreadyClaimed(1))));
    }

    #[test]
    fn premium_player_can_claim_tier_15_premium_reward() {
        let mut p = player();
        p.activate_premium().unwrap();
        p.earn_xp(15_000);
        let s = season();
        let reward = p.claim_reward(15, &s).unwrap();
        assert_eq!(reward.reward_type, RewardType::SuitSkin);
    }

    // ── unclaimed_count ───────────────────────────────────────────────────────

    #[test]
    fn unclaimed_count_equals_current_tier_when_nothing_claimed() {
        let mut p = player();
        p.earn_xp(3000);
        let s = season();
        assert_eq!(p.unclaimed_count(&s), 3);
    }

    #[test]
    fn unclaimed_count_decrements_after_claiming() {
        let mut p = player();
        p.earn_xp(2000);
        let s = season();
        p.claim_reward(1, &s).unwrap();
        assert_eq!(p.unclaimed_count(&s), 1);
    }
}
