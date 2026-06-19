//! Duel matchmaking queue.
//!
//! Players enter the ranked queue via `enqueue`.  If a waiting player is found
//! within ±200 ELO rating the two are paired immediately and an `ActiveMatch` is
//! returned.  Otherwise the new entry waits at the back of the queue.

use std::collections::{HashMap, VecDeque};

// ── Types ─────────────────────────────────────────────────────────────────────

/// A player waiting for a ranked duel partner.
#[derive(Debug, Clone)]
pub struct QueueEntry {
    pub player_id: u64,
    pub player_name: String,
    pub suit_id: String,
    /// ELO-adjacent rating used for matchmaking bracket.
    pub rating: u32,
    /// UTC timestamp when the player entered the queue.
    pub queued_at: chrono::DateTime<chrono::Utc>,
}

/// A live, server-tracked duel match.
#[derive(Debug, Clone)]
pub struct ActiveMatch {
    pub match_id: u64,
    pub player1_id: u64,
    pub player2_id: u64,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub state: MatchPhase,
}

/// Lifecycle phase of a server-side match record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchPhase {
    WaitingForPlayers,
    InProgress,
    Completed,
}

/// A direct challenge from one player to another (not a ranked queue match).
#[derive(Debug, Clone)]
pub struct DuelChallenge {
    pub id: u64,
    pub challenger_id: u64,
    pub challenged_id: u64,
    pub challenger_suit: String,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub status: ChallengeStatus,
}

/// Lifecycle state of a direct duel challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
}

// ── MatchmakingQueue ──────────────────────────────────────────────────────────

/// Server-side ranked matchmaking queue and active-match registry.
pub struct MatchmakingQueue {
    queue: VecDeque<QueueEntry>,
    active_matches: HashMap<u64, ActiveMatch>,
    next_match_id: u64,
}

impl MatchmakingQueue {
    /// Construct an empty queue.
    pub fn new() -> Self {
        MatchmakingQueue {
            queue: VecDeque::new(),
            active_matches: HashMap::new(),
            next_match_id: 1,
        }
    }

    /// Add a player to the ranked queue.
    ///
    /// If there is already a queued player whose rating is within ±200 of
    /// `entry.rating`, they are removed from the queue and an `ActiveMatch` is
    /// returned.  Otherwise `None` is returned and the entry waits.
    pub fn enqueue(&mut self, entry: QueueEntry) -> Option<ActiveMatch> {
        // Find the first queued player within ±200 rating.
        let match_partner_idx = self
            .queue
            .iter()
            .position(|q| (q.rating as i64 - entry.rating as i64).abs() <= 200);

        if let Some(idx) = match_partner_idx {
            let partner = self.queue.remove(idx).expect("index was valid");
            let match_id = self.next_match_id;
            self.next_match_id += 1;

            let active = ActiveMatch {
                match_id,
                player1_id: partner.player_id,
                player2_id: entry.player_id,
                started_at: chrono::Utc::now(),
                state: MatchPhase::InProgress,
            };
            self.active_matches.insert(match_id, active.clone());
            Some(active)
        } else {
            self.queue.push_back(entry);
            None
        }
    }

    /// Remove a player from the queue (e.g. they disconnected or cancelled).
    pub fn dequeue(&mut self, player_id: u64) {
        self.queue.retain(|e| e.player_id != player_id);
    }

    /// Number of players currently waiting for a match.
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Number of active (in-progress or waiting) matches tracked by the server.
    pub fn active_match_count(&self) -> usize {
        self.active_matches.len()
    }

    /// Look up an active match by id.
    pub fn get_match(&self, match_id: u64) -> Option<&ActiveMatch> {
        self.active_matches.get(&match_id)
    }

    /// Mark a match as completed (e.g. a winner was determined).
    pub fn complete_match(&mut self, match_id: u64) {
        if let Some(m) = self.active_matches.get_mut(&match_id) {
            m.state = MatchPhase::Completed;
        }
    }

    /// Remove a completed match from the registry.
    pub fn remove_match(&mut self, match_id: u64) {
        self.active_matches.remove(&match_id);
    }
}

impl Default for MatchmakingQueue {
    fn default() -> Self {
        Self::new()
    }
}
