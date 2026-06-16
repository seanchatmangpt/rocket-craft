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
