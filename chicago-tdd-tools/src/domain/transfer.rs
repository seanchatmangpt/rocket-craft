use crate::domain::account::Account;

/// A service for performing fund transfers between accounts.
pub struct TransferService;

impl TransferService {
    /// Transfers the specified amount from one account to another.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The amount is less than or equal to 0.
    /// - The source account has insufficient funds.
    pub fn transfer(from: &mut Account, to: &mut Account, amount: i64) -> Result<(), String> {
        if amount <= 0 {
            return Err("Transfer amount must be positive".into());
        }
        from.withdraw(amount)?;
        to.deposit(amount);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::Account;

    #[test]
    fn transfer_moves_funds_between_accounts() {
        let mut from = Account::from(200);
        let mut to = Account::from(50);
        TransferService::transfer(&mut from, &mut to, 100).unwrap();
        assert_eq!(from.balance(), 100);
        assert_eq!(to.balance(), 150);
    }

    #[test]
    fn transfer_zero_returns_error() {
        let mut from = Account::from(100);
        let mut to = Account::new();
        assert!(TransferService::transfer(&mut from, &mut to, 0).is_err());
    }

    #[test]
    fn transfer_negative_returns_error() {
        let mut from = Account::from(100);
        let mut to = Account::new();
        assert!(TransferService::transfer(&mut from, &mut to, -10).is_err());
    }

    #[test]
    fn transfer_insufficient_funds_returns_error_and_leaves_balances_unchanged() {
        let mut from = Account::from(50);
        let mut to = Account::from(100);
        let result = TransferService::transfer(&mut from, &mut to, 200);
        assert!(result.is_err());
        assert_eq!(from.balance(), 50);
        assert_eq!(to.balance(), 100);
    }

    #[test]
    fn transfer_exact_balance_drains_source() {
        let mut from = Account::from(100);
        let mut to = Account::new();
        TransferService::transfer(&mut from, &mut to, 100).unwrap();
        assert_eq!(from.balance(), 0);
        assert_eq!(to.balance(), 100);
    }
}
