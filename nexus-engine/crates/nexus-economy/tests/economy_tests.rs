use proptest::prelude::*;
use nexus_economy::{
    auction::{Auction, OpenForBids},
    ledger::{AccountType, JournalEntry, Ledger},
    marketplace::Marketplace,
};

// ── Deterministic tests ───────────────────────────────────────────────────────

#[test]
fn ledger_always_balanced_after_transfers() {
    let mut ledger = Ledger::new();
    ledger.award_gold(1, 1000, "initial grant").unwrap();
    ledger.award_gold(2, 500, "initial grant").unwrap();

    let p1 = ledger.balance_of(AccountType::PlayerWallet(1));
    let p2 = ledger.balance_of(AccountType::PlayerWallet(2));
    assert_eq!(p1, 1000);
    assert_eq!(p2, 500);

    // Transfer 200 from player 1 to player 2.
    ledger.transfer(1, 2, 200, "test transfer").unwrap();
    assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 800);
    assert_eq!(ledger.balance_of(AccountType::PlayerWallet(2)), 700);

    // Total sum of all accounts must be zero (double-entry invariant).
    assert_eq!(ledger.total_balance(), 0, "ledger must always sum to zero");
}

#[test]
fn transfer_insufficient_funds_returns_error() {
    let mut ledger = Ledger::new();
    ledger.award_gold(1, 100, "grant").unwrap();
    let result = ledger.transfer(1, 2, 200, "overspend");
    assert!(result.is_err());
    // Balance unchanged after failed transfer.
    assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 100);
    assert_eq!(ledger.total_balance(), 0);
}

#[test]
fn marketplace_buy_transfers_gold_correctly() {
    let mut ledger = Ledger::new();
    ledger.award_gold(1, 5000, "buyer grant").unwrap();
    ledger.award_gold(2, 0, "seller grant").unwrap();

    let mut market = Marketplace::new();
    let listing_id = market
        .list_item(2, "Beam Saber".to_string(), "Rare".to_string(), 1000)
        .unwrap();
    market.buy(listing_id, 1, &mut ledger).unwrap();

    assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 4000);
    assert_eq!(ledger.balance_of(AccountType::PlayerWallet(2)), 1000);
    assert_eq!(ledger.total_balance(), 0);
}

#[test]
fn marketplace_cannot_buy_own_listing() {
    let mut ledger = Ledger::new();
    ledger.award_gold(1, 5000, "grant").unwrap();

    let mut market = Marketplace::new();
    let id = market
        .list_item(1, "Shield".to_string(), "Common".to_string(), 500)
        .unwrap();
    let result = market.buy(id, 1, &mut ledger);
    assert!(result.is_err());
}

#[test]
fn marketplace_zero_price_rejected() {
    let mut market = Marketplace::new();
    let result = market.list_item(1, "Junk".to_string(), "Common".to_string(), 0);
    assert!(result.is_err());
}

#[test]
fn auction_bid_below_starting_price_rejected() {
    let mut ledger = Ledger::new();
    ledger.award_gold(1, 10_000, "bidder gold").unwrap();

    let mut auction =
        Auction::<OpenForBids>::new(1, 99, "Nu Gundam".to_string(), 500, None, 24);
    let result = auction.place_bid(1, 400, &mut ledger);
    assert!(result.is_err());
    assert_eq!(ledger.total_balance(), 0);
}

#[test]
fn auction_seller_cannot_bid() {
    let mut ledger = Ledger::new();
    ledger.award_gold(42, 10_000, "seller gold").unwrap();

    let mut auction =
        Auction::<OpenForBids>::new(1, 42, "Zaku II".to_string(), 100, None, 24);
    let result = auction.place_bid(42, 200, &mut ledger);
    assert!(result.is_err());
}

#[test]
fn auction_happy_path_conserves_gold() {
    let mut ledger = Ledger::new();
    ledger.award_gold(1, 10_000, "bidder gold").unwrap();

    let seller_id = 99_u64;
    // Seller starts with 0 gold (they give item, receive gold)
    let mut auction =
        Auction::<OpenForBids>::new(1, seller_id, "Gundam Wing".to_string(), 1000, None, 24);
    auction.place_bid(1, 1000, &mut ledger).unwrap();
    auction.close(&mut ledger).unwrap();

    // Total balance still zero after settlement.
    assert_eq!(ledger.total_balance(), 0);
    // Seller received 950 (5 % fee = 50)
    assert_eq!(
        ledger.balance_of(AccountType::PlayerWallet(seller_id)),
        950
    );
    // Bidder spent 1000
    assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 9_000);
    // AuctionHouse received 50 (fee)
    assert_eq!(ledger.balance_of(AccountType::AuctionHouse), 50);
}

#[test]
fn auction_reserve_not_met_refunds_bidder() {
    let mut ledger = Ledger::new();
    ledger.award_gold(1, 10_000, "bidder gold").unwrap();

    let mut auction = Auction::<OpenForBids>::new(
        1,
        99,
        "Sazabi".to_string(),
        100,
        Some(5000), // reserve much higher than any bid
        24,
    );
    auction.place_bid(1, 100, &mut ledger).unwrap();
    auction.close(&mut ledger).unwrap();

    // Bidder should be fully refunded.
    assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 10_000);
    assert_eq!(ledger.total_balance(), 0);
}

#[test]
fn auction_outbid_refunds_previous_bidder() {
    let mut ledger = Ledger::new();
    ledger.award_gold(1, 10_000, "bidder 1 gold").unwrap();
    ledger.award_gold(2, 10_000, "bidder 2 gold").unwrap();

    let mut auction =
        Auction::<OpenForBids>::new(1, 99, "Destiny Gundam".to_string(), 100, None, 24);

    auction.place_bid(1, 100, &mut ledger).unwrap();
    // At this point player 1 has 9900 (100 in escrow).
    assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 9_900);

    // Player 2 outbids — player 1 should be refunded automatically.
    let min_bid = 100 + (100 / 20).max(1); // 105
    auction.place_bid(2, min_bid, &mut ledger).unwrap();
    assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 10_000); // refunded

    assert_eq!(ledger.total_balance(), 0);
}

// ── Property-based tests ──────────────────────────────────────────────────────

proptest! {
    /// Any sequence of transfers preserves total balance at zero.
    #[test]
    fn gold_conservation_invariant(
        transfers in prop::collection::vec((0u64..5, 0u64..5, 1u32..1000), 1..20)
    ) {
        let mut ledger = Ledger::new();
        // Give all 5 players starting gold.
        for player_id in 0..5u64 {
            ledger.award_gold(player_id, 10_000, "starting gold").unwrap();
        }

        for (from, to, amount) in &transfers {
            // Ignore errors (insufficient funds etc.) — just ensure invariant holds.
            let _ = ledger.transfer(*from, *to, *amount, "test");
            prop_assert_eq!(ledger.total_balance(), 0, "double-entry must always balance");
        }
    }

    /// Marketplace: seller receives what buyer pays; total balance stays zero.
    #[test]
    fn marketplace_gold_conservation(price in 1u32..10_000) {
        let mut ledger = Ledger::new();
        ledger.award_gold(1, price + 1000, "buyer starting gold").unwrap();
        // Player 2 (seller) starts at zero — award 0 is a no-op, but keeps symmetry.
        ledger.award_gold(2, 0, "seller starting gold").unwrap();

        let mut market = Marketplace::new();
        let listing_id = market
            .list_item(2, "Beam Saber".to_string(), "Rare".to_string(), price)
            .unwrap();
        market.buy(listing_id, 1, &mut ledger).unwrap();

        // Seller received exactly `price`, ledger still balanced.
        prop_assert_eq!(ledger.total_balance(), 0);
        prop_assert_eq!(ledger.balance_of(AccountType::PlayerWallet(2)), price as i64);
    }

    /// Unbalanced journal entries are rejected.
    #[test]
    fn unbalanced_entry_rejected(debit in 1u32..1000, credit in 1u32..1000) {
        prop_assume!(debit != credit);
        let mut ledger = Ledger::new();
        let result = ledger.record(JournalEntry {
            id: 0,
            timestamp: chrono::Utc::now(),
            description: "bad entry".to_string(),
            debits: vec![(AccountType::SystemSource, debit)],
            credits: vec![(AccountType::PlayerWallet(1), credit)],
            transaction_id: 0,
        });
        prop_assert!(result.is_err());
    }

    /// Auction minimum bid enforcement.
    #[test]
    fn auction_bid_below_minimum_rejected(starting in 100u32..1000) {
        let mut ledger = Ledger::new();
        ledger.award_gold(1, 10000, "bidder gold").unwrap();
        let mut auction =
            Auction::<OpenForBids>::new(1, 99, "Nu Gundam".to_string(), starting, None, 24);
        // Bid below starting price should fail.
        let result = auction.place_bid(1, starting - 1, &mut ledger);
        prop_assert!(result.is_err());
        // No gold should have moved.
        prop_assert_eq!(ledger.balance_of(AccountType::PlayerWallet(1)), 10_000);
        prop_assert_eq!(ledger.total_balance(), 0);
    }

    /// After any number of valid auction bids, the ledger always balances.
    #[test]
    fn auction_multi_bid_gold_conservation(
        bids in prop::collection::vec((1u64..5, 100u32..500), 1..10)
    ) {
        let mut ledger = Ledger::new();
        // Give bidders ample gold.
        for player_id in 1u64..=5 {
            ledger.award_gold(player_id, 1_000_000, "test grant").unwrap();
        }

        let seller_id = 99u64;
        let mut auction =
            Auction::<OpenForBids>::new(1, seller_id, "Kampfer".to_string(), 100, None, 24);

        for (bidder_id, amount) in &bids {
            // Ignore errors (bid too low, etc.) — invariant must still hold.
            let _ = auction.place_bid(*bidder_id, *amount, &mut ledger);
            prop_assert_eq!(ledger.total_balance(), 0);
        }

        auction.close(&mut ledger).unwrap();
        prop_assert_eq!(ledger.total_balance(), 0);
    }
}
