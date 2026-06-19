//! # Account Module
//!
//! Provides the `Account` struct representing bank accounts with balance manipulation operations.

use crate::cli::ClapNoun;
use anyhow::Result;
use clap::Subcommand;

/// Represents a bank account with a balance.
///
/// This structure provides basic operations for managing funds, including
/// deposits, withdrawals, and balance inquiries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Account {
    balance: i64,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AccountVerb {
    /// Deposit money into the account
    Deposit { amount: i64 },
    /// Withdraw money from the account
    Withdraw { amount: i64 },
    /// Check the current balance
    Balance,
}

impl ClapNoun for Account {
    type Verb = AccountVerb;

    fn handle(&mut self, verb: Self::Verb) -> Result<()> {
        match verb {
            AccountVerb::Deposit { amount } => {
                self.deposit(amount);
            }
            AccountVerb::Withdraw { amount } => {
                self.withdraw(amount).map_err(|e| anyhow::anyhow!(e))?;
            }
            AccountVerb::Balance => {
                tracing::info!("Current balance: {}", self.balance);
            }
        }
        Ok(())
    }
}

impl From<i64> for Account {
    fn from(balance: i64) -> Self {
        Self { balance }
    }
}

impl Account {
    /// Creates a new `Account` with an initial balance of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use chicago_tdd_tools::Account;
    /// let mut acc = Account::new();
    /// assert_eq!(acc.balance(), 0);
    /// acc.deposit(50);
    /// assert_eq!(acc.balance(), 50);
    /// ```
    pub fn new() -> Self {
        Self { balance: 0 }
    }

    /// Deposits the specified amount into the account.
    ///
    /// If the amount is less than or equal to 0, the operation is ignored.
    pub fn deposit(&mut self, amount: i64) {
        if amount > 0 {
            self.balance += amount;
        }
    }

    /// Withdraws the specified amount from the account.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The amount is negative.
    /// - There are insufficient funds in the account.
    pub fn withdraw(&mut self, amount: i64) -> Result<(), String> {
        if amount < 0 {
            return Err("Cannot withdraw negative amount".into());
        }
        if amount > self.balance {
            return Err("Insufficient funds".into());
        }
        self.balance -= amount;
        Ok(())
    }

    /// Returns the current balance of the account.
    pub fn balance(&self) -> i64 {
        self.balance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_account_has_zero_balance() {
        assert_eq!(Account::new().balance(), 0);
    }

    #[test]
    fn deposit_positive_increases_balance() {
        let mut a = Account::new();
        a.deposit(100);
        assert_eq!(a.balance(), 100);
    }

    #[test]
    fn deposit_zero_is_ignored() {
        let mut a = Account::new();
        a.deposit(0);
        assert_eq!(a.balance(), 0);
    }

    #[test]
    fn deposit_negative_is_ignored() {
        let mut a = Account::new();
        a.deposit(-50);
        assert_eq!(a.balance(), 0);
    }

    #[test]
    fn withdraw_reduces_balance() {
        let mut a = Account::from(200);
        a.withdraw(80).unwrap();
        assert_eq!(a.balance(), 120);
    }

    #[test]
    fn withdraw_exact_balance_leaves_zero() {
        let mut a = Account::from(100);
        a.withdraw(100).unwrap();
        assert_eq!(a.balance(), 0);
    }

    #[test]
    fn withdraw_insufficient_funds_returns_error() {
        let mut a = Account::from(50);
        let e = a.withdraw(100).unwrap_err();
        assert!(e.contains("Insufficient"));
        assert_eq!(a.balance(), 50); // unchanged
    }

    #[test]
    fn withdraw_negative_amount_returns_error() {
        let mut a = Account::from(100);
        assert!(a.withdraw(-10).is_err());
        assert_eq!(a.balance(), 100); // unchanged
    }

    #[test]
    fn account_from_i64_sets_balance() {
        let a = Account::from(500);
        assert_eq!(a.balance(), 500);
    }

    #[test]
    fn multiple_deposits_accumulate() {
        let mut a = Account::new();
        a.deposit(30);
        a.deposit(70);
        assert_eq!(a.balance(), 100);
    }
}
