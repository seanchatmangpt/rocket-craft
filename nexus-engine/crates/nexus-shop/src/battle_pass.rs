use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

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

    pub fn claim_reward(&mut self, tier: u32, season: &BattlePassSeason) -> Result<PassReward, BattlePassError> {
        if tier > self.current_tier {
            return Err(BattlePassError::TierNotReached { tier, current: self.current_tier });
        }
        if self.claimed_tiers.contains(&tier) {
            return Err(BattlePassError::AlreadyClaimed(tier));
        }

        let rewards = if tier > 10 || self.is_premium {
            &season.premium_rewards
        } else {
            &season.free_rewards
        };

        let reward = rewards.iter()
            .find(|r| r.tier == tier)
            .ok_or(BattlePassError::RewardNotFound(tier))?
            .clone();

        self.claimed_tiers.push(tier);
        Ok(reward)
    }

    pub fn unclaimed_count(&self, _season: &BattlePassSeason) -> usize {
        let available: Vec<u32> = (1..=self.current_tier).collect();
        available.iter().filter(|t| !self.claimed_tiers.contains(t)).count()
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
