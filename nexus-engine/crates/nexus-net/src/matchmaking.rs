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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn entry(player_id: u64, rating: u32) -> QueueEntry {
        QueueEntry {
            player_id,
            player_name: format!("pilot-{player_id}"),
            suit_id: "rx78".into(),
            rating,
            queued_at: Utc::now(),
        }
    }

    // ── enqueue — no match ────────────────────────────────────────────────────

    #[test]
    fn single_entry_waits_in_queue() {
        let mut q = MatchmakingQueue::new();
        let result = q.enqueue(entry(1, 1000));
        assert!(result.is_none(), "no match when queue is empty");
        assert_eq!(q.queue_len(), 1);
    }

    #[test]
    fn players_outside_200_elo_do_not_match() {
        let mut q = MatchmakingQueue::new();
        q.enqueue(entry(1, 1000));
        let result = q.enqueue(entry(2, 1201)); // 201 apart
        assert!(result.is_none());
        assert_eq!(q.queue_len(), 2);
    }

    // ── enqueue — match found ─────────────────────────────────────────────────

    #[test]
    fn players_within_200_elo_produce_active_match() {
        let mut q = MatchmakingQueue::new();
        q.enqueue(entry(1, 1000));
        let m = q.enqueue(entry(2, 1200)).expect("should match at boundary");
        assert_eq!(m.player1_id, 1);
        assert_eq!(m.player2_id, 2);
        assert_eq!(m.state, MatchPhase::InProgress);
    }

    #[test]
    fn match_removes_partner_from_queue() {
        let mut q = MatchmakingQueue::new();
        q.enqueue(entry(1, 1000));
        q.enqueue(entry(2, 1000)); // matches with 1
        assert_eq!(q.queue_len(), 0, "matched partner must leave queue");
    }

    #[test]
    fn match_is_registered_in_active_matches() {
        let mut q = MatchmakingQueue::new();
        q.enqueue(entry(1, 1000));
        let m = q.enqueue(entry(2, 1000)).unwrap();
        assert_eq!(q.active_match_count(), 1);
        assert!(q.get_match(m.match_id).is_some());
    }

    #[test]
    fn match_ids_increment_monotonically() {
        let mut q = MatchmakingQueue::new();
        q.enqueue(entry(1, 1000));
        let m1 = q.enqueue(entry(2, 1000)).unwrap();
        q.enqueue(entry(3, 1000));
        let m2 = q.enqueue(entry(4, 1000)).unwrap();
        assert!(m2.match_id > m1.match_id);
    }

    #[test]
    fn exact_200_elo_gap_still_matches() {
        let mut q = MatchmakingQueue::new();
        q.enqueue(entry(1, 800));
        let m = q.enqueue(entry(2, 1000)); // exactly 200 apart
        assert!(m.is_some(), "|800-1000| = 200 is within ±200");
    }

    // ── dequeue ───────────────────────────────────────────────────────────────

    #[test]
    fn dequeue_removes_player_from_waiting_queue() {
        let mut q = MatchmakingQueue::new();
        q.enqueue(entry(1, 1000));
        q.enqueue(entry(2, 1500)); // different bracket, no match
        q.dequeue(1);
        assert_eq!(q.queue_len(), 1);
    }

    #[test]
    fn dequeue_nonexistent_player_is_noop() {
        let mut q = MatchmakingQueue::new();
        q.dequeue(99); // no panic
        assert_eq!(q.queue_len(), 0);
    }

    // ── complete_match / remove_match ─────────────────────────────────────────

    #[test]
    fn complete_match_sets_state_to_completed() {
        let mut q = MatchmakingQueue::new();
        q.enqueue(entry(1, 1000));
        let m = q.enqueue(entry(2, 1000)).unwrap();
        q.complete_match(m.match_id);
        assert_eq!(
            q.get_match(m.match_id).unwrap().state,
            MatchPhase::Completed
        );
    }

    #[test]
    fn remove_match_deletes_from_registry() {
        let mut q = MatchmakingQueue::new();
        q.enqueue(entry(1, 1000));
        let m = q.enqueue(entry(2, 1000)).unwrap();
        q.remove_match(m.match_id);
        assert!(q.get_match(m.match_id).is_none());
        assert_eq!(q.active_match_count(), 0);
    }
}
