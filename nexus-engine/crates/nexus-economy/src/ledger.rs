use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountType {
    /// A player's gold balance, keyed by player ID
    PlayerWallet(u64),
    /// Shop income
    ShopRevenue,
    /// Auction escrow (player ID 0 acts as the escrow account in transfers)
    AuctionHouse,
    /// Gacha revenue
    GachaPool,
    /// Battle pass income
    BattlePassRevenue,
    /// Gold removed from economy (death, fees)
    SystemSink,
    /// Gold injected (rewards, login bonuses)
    SystemSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    /// (account, amount) — accounts being debited
    pub debits: Vec<(AccountType, u32)>,
    /// (account, amount) — accounts being credited
    pub credits: Vec<(AccountType, u32)>,
    pub transaction_id: u64,
}

impl JournalEntry {
    /// Validates that total debits == total credits (double-entry invariant).
    pub fn is_balanced(&self) -> bool {
        let total_debits: u32 = self.debits.iter().map(|(_, a)| a).sum();
        let total_credits: u32 = self.credits.iter().map(|(_, a)| a).sum();
        total_debits == total_credits
    }
}

/// Double-entry accounting ledger.  Every recorded entry must balance
/// (debits == credits), so the sum of all account balances is always zero.
pub struct Ledger {
    entries: Vec<JournalEntry>,
    /// i64 lets us detect sign errors that would be hidden by u64 wrapping.
    balances: HashMap<AccountType, i64>,
    next_entry_id: u64,
    next_tx_id: u64,
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            balances: HashMap::new(),
            next_entry_id: 0,
            next_tx_id: 0,
        }
    }

    /// Allocate and return the next transaction ID (increments the counter).
    pub fn next_tx_id(&mut self) -> u64 {
        let id = self.next_tx_id;
        self.next_tx_id += 1;
        id
    }

    /// Allocate and return the next journal-entry ID (increments the counter).
    pub fn next_entry_id(&mut self) -> u64 {
        let id = self.next_entry_id;
        self.next_entry_id += 1;
        id
    }

    /// Record a balanced transaction.  Returns the entry id on success, or
    /// `LedgerError::UnbalancedEntry` if debits != credits.
    pub fn record(&mut self, entry: JournalEntry) -> Result<u64, LedgerError> {
        if !entry.is_balanced() {
            let debits: u32 = entry.debits.iter().map(|(_, a)| a).sum();
            let credits: u32 = entry.credits.iter().map(|(_, a)| a).sum();
            return Err(LedgerError::UnbalancedEntry { debits, credits });
        }

        // Apply balance changes: debits reduce a balance, credits increase it.
        for (acct, amount) in &entry.debits {
            *self.balances.entry(*acct).or_insert(0) -= *amount as i64;
        }
        for (acct, amount) in &entry.credits {
            *self.balances.entry(*acct).or_insert(0) += *amount as i64;
        }

        let id = entry.id;
        self.entries.push(entry);
        Ok(id)
    }

    /// Current balance of an account (positive = net credit, negative = net debit).
    pub fn balance_of(&self, account: AccountType) -> i64 {
        self.balances.get(&account).copied().unwrap_or(0)
    }

    /// The fundamental double-entry invariant: all balances must sum to zero.
    pub fn total_balance(&self) -> i64 {
        self.balances.values().sum()
    }

    /// Atomically transfer `amount` gold from `from` to `to`.
    ///
    /// Returns `LedgerError::InsufficientFunds` if `from` does not have enough
    /// gold.  Note: player wallet 0 is used as the auction escrow account;
    /// callers that need to release from escrow should use `record` directly.
    pub fn transfer(
        &mut self,
        from: u64,
        to: u64,
        amount: u32,
        reason: &str,
    ) -> Result<u64, LedgerError> {
        if self.balance_of(AccountType::PlayerWallet(from)) < amount as i64 {
            return Err(LedgerError::InsufficientFunds {
                player_id: from,
                needed: amount,
            });
        }

        let tx_id = self.next_tx_id();
        let entry_id = self.next_entry_id();

        self.record(JournalEntry {
            id: entry_id,
            timestamp: Utc::now(),
            description: reason.to_string(),
            debits: vec![(AccountType::PlayerWallet(from), amount)],
            credits: vec![(AccountType::PlayerWallet(to), amount)],
            transaction_id: tx_id,
        })
    }

    /// Award `amount` gold to `player_id`, sourced from the system.
    pub fn award_gold(
        &mut self,
        player_id: u64,
        amount: u32,
        reason: &str,
    ) -> Result<u64, LedgerError> {
        let tx_id = self.next_tx_id();
        let entry_id = self.next_entry_id();

        self.record(JournalEntry {
            id: entry_id,
            timestamp: Utc::now(),
            description: reason.to_string(),
            debits: vec![(AccountType::SystemSource, amount)],
            credits: vec![(AccountType::PlayerWallet(player_id), amount)],
            transaction_id: tx_id,
        })
    }

    /// Full journal history in insertion order.
    pub fn history(&self) -> &[JournalEntry] {
        &self.entries
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    #[error("unbalanced entry: debits={debits} != credits={credits}")]
    UnbalancedEntry { debits: u32, credits: u32 },

    #[error("insufficient funds: player {player_id} needs {needed} gold")]
    InsufficientFunds { player_id: u64, needed: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(
        ledger: &mut Ledger,
        debits: Vec<(AccountType, u32)>,
        credits: Vec<(AccountType, u32)>,
    ) -> JournalEntry {
        JournalEntry {
            id: ledger.next_entry_id(),
            timestamp: Utc::now(),
            description: "test".into(),
            debits,
            credits,
            transaction_id: ledger.next_tx_id(),
        }
    }

    // ── JournalEntry::is_balanced ─────────────────────────────────────────────

    #[test]
    fn balanced_entry_passes() {
        let entry = JournalEntry {
            id: 0,
            timestamp: Utc::now(),
            description: "".into(),
            transaction_id: 0,
            debits: vec![(AccountType::PlayerWallet(1), 100)],
            credits: vec![(AccountType::PlayerWallet(2), 100)],
        };
        assert!(entry.is_balanced());
    }

    #[test]
    fn unbalanced_entry_fails_is_balanced() {
        let entry = JournalEntry {
            id: 0,
            timestamp: Utc::now(),
            description: "".into(),
            transaction_id: 0,
            debits: vec![(AccountType::PlayerWallet(1), 100)],
            credits: vec![(AccountType::PlayerWallet(2), 50)],
        };
        assert!(!entry.is_balanced());
    }

    // ── Ledger::record ────────────────────────────────────────────────────────

    #[test]
    fn unbalanced_entry_rejected_with_error() {
        let mut ledger = Ledger::new();
        let entry = make_entry(
            &mut ledger,
            vec![(AccountType::PlayerWallet(1), 100)],
            vec![(AccountType::PlayerWallet(2), 50)], // mismatch
        );
        assert!(matches!(
            ledger.record(entry),
            Err(LedgerError::UnbalancedEntry { .. })
        ));
    }

    #[test]
    fn total_balance_is_always_zero_after_valid_entries() {
        let mut ledger = Ledger::new();
        // Source → player
        let e1 = make_entry(
            &mut ledger,
            vec![(AccountType::SystemSource, 500)],
            vec![(AccountType::PlayerWallet(1), 500)],
        );
        ledger.record(e1).unwrap();
        assert_eq!(ledger.total_balance(), 0, "double-entry invariant");

        // Player → shop
        let e2 = make_entry(
            &mut ledger,
            vec![(AccountType::PlayerWallet(1), 100)],
            vec![(AccountType::ShopRevenue, 100)],
        );
        ledger.record(e2).unwrap();
        assert_eq!(
            ledger.total_balance(),
            0,
            "double-entry invariant after shop purchase"
        );
    }

    #[test]
    fn balance_of_reflects_credits_and_debits() {
        let mut ledger = Ledger::new();
        let e = make_entry(
            &mut ledger,
            vec![(AccountType::SystemSource, 1000)],
            vec![(AccountType::PlayerWallet(42), 1000)],
        );
        ledger.record(e).unwrap();
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(42)), 1000);
        assert_eq!(ledger.balance_of(AccountType::SystemSource), -1000);
    }

    // ── Ledger::transfer ──────────────────────────────────────────────────────

    #[test]
    fn transfer_moves_gold_between_players() {
        let mut ledger = Ledger::new();
        // Fund player 1
        let e = make_entry(
            &mut ledger,
            vec![(AccountType::SystemSource, 500)],
            vec![(AccountType::PlayerWallet(1), 500)],
        );
        ledger.record(e).unwrap();

        ledger.transfer(1, 2, 200, "trade").unwrap();
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 300);
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(2)), 200);
        assert_eq!(ledger.total_balance(), 0);
    }

    #[test]
    fn transfer_insufficient_funds_rejected() {
        let mut ledger = Ledger::new();
        let err = ledger.transfer(99, 1, 100, "empty wallet").unwrap_err();
        assert!(matches!(
            err,
            LedgerError::InsufficientFunds {
                player_id: 99,
                needed: 100
            }
        ));
    }

    // ── Ledger::award_gold ────────────────────────────────────────────────────

    #[test]
    fn award_gold_credits_player_and_debits_source() {
        let mut ledger = Ledger::new();
        ledger.award_gold(7, 250, "login bonus").unwrap();
        assert_eq!(ledger.balance_of(AccountType::PlayerWallet(7)), 250);
        assert_eq!(ledger.balance_of(AccountType::SystemSource), -250);
        assert_eq!(ledger.total_balance(), 0);
    }

    #[test]
    fn history_records_all_entries_in_order() {
        let mut ledger = Ledger::new();
        ledger.award_gold(1, 100, "r1").unwrap();
        ledger.award_gold(2, 200, "r2").unwrap();
        let hist = ledger.history();
        assert_eq!(hist.len(), 2);
        assert_eq!(hist[0].description, "r1");
        assert_eq!(hist[1].description, "r2");
    }
}
