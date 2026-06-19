use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    pub id: i64,
    pub name: String,
    pub score: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaderboardEntry {
    pub id: i64,
    pub player_id: i64,
    pub score: i64,
    pub rank: i64,
}

pub struct SupabaseService {
    client: Client,
    url: String,
    anon_key: String,
}

impl SupabaseService {
    pub fn new(url: String, anon_key: String) -> Self {
        Self {
            client: Client::new(),
            url,
            anon_key,
        }
    }

    pub async fn get_players(&self) -> Result<Vec<Player>> {
        let response = self
            .client
            .get(format!("{}/rest/v1/players?select=*", self.url))
            .header("apikey", &self.anon_key)
            .header("Authorization", &format!("Bearer {}", self.anon_key))
            .send()
            .await?;

        response.error_for_status_ref()?;

        let players = response.json::<Vec<Player>>().await?;
        Ok(players)
    }

    pub async fn get_leaderboard(&self) -> Result<Vec<LeaderboardEntry>> {
        let response = self
            .client
            .get(format!("{}/rest/v1/leaderboard?select=*", self.url))
            .header("apikey", &self.anon_key)
            .header("Authorization", &format!("Bearer {}", self.anon_key))
            .send()
            .await?;

        response.error_for_status_ref()?;

        let leaderboard = response.json::<Vec<LeaderboardEntry>>().await?;
        Ok(leaderboard)
    }
}
