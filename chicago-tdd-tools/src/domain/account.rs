use clap::Subcommand;
use crate::cli::ClapNoun;
use anyhow::Result;

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
                println!("Current balance: {}", self.balance);
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
