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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn supabase_service_new_stores_url_and_key() {
        let svc = SupabaseService::new("http://localhost:54321".into(), "my-key".into());
        assert_eq!(svc.url, "http://localhost:54321");
        assert_eq!(svc.anon_key, "my-key");
    }

    #[test]
    fn player_deserializes_from_json() {
        let p: Player = serde_json::from_value(json!({
            "id": 1, "name": "Alice", "score": 9001
        })).unwrap();
        assert_eq!(p.id, 1);
        assert_eq!(p.name, "Alice");
        assert_eq!(p.score, 9001);
    }

    #[test]
    fn player_serializes_to_json() {
        let p = Player { id: 2, name: "Bob".into(), score: 42 };
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["name"], "Bob");
        assert_eq!(v["score"], 42);
    }

    #[test]
    fn leaderboard_entry_roundtrips() {
        let entry = LeaderboardEntry { id: 1, player_id: 7, score: 500, rank: 3 };
        let json = serde_json::to_string(&entry).unwrap();
        let back: LeaderboardEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.player_id, 7);
        assert_eq!(back.rank, 3);
    }

    #[test]
    fn player_debug_format_contains_name() {
        let p = Player { id: 1, name: "Tester".into(), score: 0 };
        assert!(format!("{:?}", p).contains("Tester"));
    }
}
