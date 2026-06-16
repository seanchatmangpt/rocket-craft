//! Integration-level assertion helpers.
//!
//! These panicking helpers produce informative messages when integration
//! invariants are violated, making test failures easier to diagnose.

use crate::fixtures::{ReceiptChain, OcelLog, EventLog, validate_ocel};

/// Assert that every receipt in the chain has a non-empty key and a non-empty
/// hash, and that the chain is not empty.
pub fn assert_chain_valid(chain: &ReceiptChain) {
    assert!(!chain.is_empty(), "ReceiptChain must not be empty");
    for (i, receipt) in chain.receipts().iter().enumerate() {
        assert!(
            !receipt.key.is_empty(),
            "Receipt at index {} has an empty key",
            i
        );
        assert!(
            !receipt.hash.is_empty(),
            "Receipt at index {} has an empty hash",
            i
        );
        assert!(
            receipt.issued_at > 0,
            "Receipt at index {} has issued_at == 0",
            i
        );
    }
}

/// Assert that an OCEL log has at least one object and at least one event,
/// and that no events reference unknown objects.
pub fn assert_ocel_valid(log: &OcelLog) {
    assert!(!log.objects.is_empty(), "OcelLog must have at least one object");
    assert!(!log.events.is_empty(), "OcelLog must have at least one event");

    let violations = validate_ocel(log);
    assert!(
        violations.is_empty(),
        "OcelLog has dangling references: {:?}",
        violations
    );
}

/// Assert that the chain contains exactly `expected` receipts.
pub fn assert_receipt_count(chain: &ReceiptChain, expected: usize) {
    assert_eq!(
        chain.len(),
        expected,
        "Expected {} receipts in chain, found {}",
        expected,
        chain.len()
    );
}

/// Assert that an event log has no violations (non-empty traces, events with
/// non-empty names, and monotone timestamps within each trace).
pub fn assert_no_pm_violations(log: &EventLog) {
    assert!(!log.traces.is_empty(), "EventLog must have at least one trace");
    for trace in &log.traces {
        assert!(
            !trace.case_id.is_empty(),
            "Trace has an empty case_id"
        );
        for event in &trace.events {
            assert!(
                !event.name.is_empty(),
                "Event in trace '{}' has an empty name",
                trace.case_id
            );
        }
        // Check timestamps are non-decreasing
        for window in trace.events.windows(2) {
            assert!(
                window[0].timestamp <= window[1].timestamp,
                "Trace '{}' has non-monotone timestamps: {} > {}",
                trace.case_id,
                window[0].timestamp,
                window[1].timestamp
            );
        }
    }
}
