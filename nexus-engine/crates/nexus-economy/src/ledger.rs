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
