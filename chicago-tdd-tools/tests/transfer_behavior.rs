use chicago_tdd_tools::{Account, TransferService};

#[test]
fn should_transfer_funds_between_accounts() {
    let mut sender = Account::new();
    let mut receiver = Account::new();
    sender.deposit(100);

    TransferService::transfer(&mut sender, &mut receiver, 40).expect("Transfer should succeed");

    assert_eq!(sender.balance(), 60);
    assert_eq!(receiver.balance(), 40);
}

#[test]
fn should_fail_transfer_if_sender_has_insufficient_funds() {
    let mut sender = Account::new();
    let mut receiver = Account::new();
    sender.deposit(30);

    let result = TransferService::transfer(&mut sender, &mut receiver, 40);

    assert!(result.is_err());
    assert_eq!(sender.balance(), 30);
    assert_eq!(receiver.balance(), 0);
}

#[test]
fn should_fail_transfer_with_negative_amount() {
    let mut sender = Account::new();
    let mut receiver = Account::new();
    sender.deposit(100);

    let result = TransferService::transfer(&mut sender, &mut receiver, -10);

    assert!(result.is_err());
    assert_eq!(sender.balance(), 100);
    assert_eq!(receiver.balance(), 0);
}
