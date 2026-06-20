//! Integration-level assertion helpers.
//!
//! These panicking helpers produce informative messages when integration
//! invariants are violated, making test failures easier to diagnose.

use crate::fixtures::{validate_ocel, EventLog, OcelLog, ReceiptChain};

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
    assert!(
        !log.objects.is_empty(),
        "OcelLog must have at least one object"
    );
    assert!(
        !log.events.is_empty(),
        "OcelLog must have at least one event"
    );

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
    assert!(
        !log.traces.is_empty(),
        "EventLog must have at least one trace"
    );
    for trace in &log.traces {
        assert!(!trace.case_id.is_empty(), "Trace has an empty case_id");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{Event, OcelEvent, OcelObject, ReceiptChain, Trace};
    use unify_receipts::receipt::Receipt;

    fn make_chain(count: usize) -> ReceiptChain {
        let mut c = ReceiptChain::new();
        for i in 0..count {
            c.append(Receipt::new(format!("key-{i}"), b"data"));
        }
        c
    }

    fn make_ocel_log() -> OcelLog {
        OcelLog {
            objects: vec![OcelObject { id: "obj1".into(), object_type: "Case".into() }],
            events: vec![OcelEvent {
                id: "e1".into(),
                event_type: "start".into(),
                related_object_ids: vec!["obj1".into()],
                timestamp: 1000,
            }],
        }
    }

    fn make_event_log(monotone: bool) -> EventLog {
        EventLog {
            traces: vec![Trace {
                case_id: "c1".into(),
                events: if monotone {
                    vec![
                        Event { name: "a".into(), timestamp: 10 },
                        Event { name: "b".into(), timestamp: 20 },
                    ]
                } else {
                    vec![
                        Event { name: "a".into(), timestamp: 20 },
                        Event { name: "b".into(), timestamp: 10 },
                    ]
                },
            }],
        }
    }

    #[test]
    fn assert_chain_valid_passes_on_good_chain() {
        assert_chain_valid(&make_chain(2));
    }

    #[test]
    #[should_panic(expected = "must not be empty")]
    fn assert_chain_valid_panics_on_empty_chain() {
        assert_chain_valid(&ReceiptChain::new());
    }

    #[test]
    fn assert_receipt_count_passes_when_correct() {
        assert_receipt_count(&make_chain(3), 3);
    }

    #[test]
    #[should_panic(expected = "Expected 2 receipts")]
    fn assert_receipt_count_panics_on_mismatch() {
        assert_receipt_count(&make_chain(3), 2);
    }

    #[test]
    fn assert_ocel_valid_passes_on_good_log() {
        assert_ocel_valid(&make_ocel_log());
    }

    #[test]
    fn assert_no_pm_violations_passes_on_monotone_log() {
        assert_no_pm_violations(&make_event_log(true));
    }

    #[test]
    #[should_panic]
    fn assert_no_pm_violations_panics_on_non_monotone() {
        assert_no_pm_violations(&make_event_log(false));
    }
}
