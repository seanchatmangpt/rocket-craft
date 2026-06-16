use chicago_tdd_tools::Account;

#[test]
fn should_start_with_zero_balance() {
    let account = Account::new();
    assert_eq!(account.balance(), 0);
}

#[test]
fn should_increase_balance_on_deposit() {
    let mut account = Account::new();
    account.deposit(100);
    assert_eq!(account.balance(), 100);
}

#[test]
fn should_decrease_balance_on_withdrawal() {
    let mut account = Account::new();
    account.deposit(100);
    account.withdraw(40).expect("Withdrawal should succeed");
    assert_eq!(account.balance(), 60);
}

#[test]
fn should_fail_withdrawal_if_insufficient_funds() {
    let mut account = Account::new();
    account.deposit(50);
    let result = account.withdraw(60);
    assert!(result.is_err());
    assert_eq!(account.balance(), 50); // Balance should remain unchanged
}

#[test]
fn should_ignore_negative_deposits() {
    let mut account = Account::new();
    account.deposit(-50);
    assert_eq!(account.balance(), 0);
}
