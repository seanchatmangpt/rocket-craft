use chicago_tdd_tools::{Logger, TuiBufferSink};
use proptest::prelude::*;
use wasm_patterns::{ActorMailbox, ActorMessage, ActorSystem};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _buffer) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[derive(Debug, Clone)]
struct PingMsg {
    value: u32,
}
impl ActorMessage for PingMsg {}

#[test]
fn actor_mailbox_delivers_messages_in_fifo_order() {
    let log = log();
    log.info("Given an ActorMailbox with capacity 10");
    let mut mb = ActorMailbox::<PingMsg>::new(0, 10);

    log.info("When three messages are sent");
    mb.send(PingMsg { value: 1 }).unwrap();
    mb.send(PingMsg { value: 2 }).unwrap();
    mb.send(PingMsg { value: 3 }).unwrap();

    log.info("Then they are received in insertion order (FIFO)");
    assert_eq!(mb.receive().unwrap().value, 1);
    assert_eq!(mb.receive().unwrap().value, 2);
    assert_eq!(mb.receive().unwrap().value, 3);
    assert!(mb.receive().is_none());
}

#[test]
fn actor_mailbox_enforces_capacity() {
    let log = log();
    log.info("Given an ActorMailbox with capacity 2");
    let mut mb = ActorMailbox::<PingMsg>::new(0, 2);

    log.info("When the mailbox is filled to capacity");
    mb.send(PingMsg { value: 1 }).unwrap();
    mb.send(PingMsg { value: 2 }).unwrap();

    log.info("Then sending another message returns an error and is_full is true");
    assert!(mb.send(PingMsg { value: 3 }).is_err());
    assert!(mb.is_full());
}

#[test]
fn actor_system_assigns_workers_by_affinity() {
    let log = log();
    log.info("Given an ActorSystem with 4 workers");
    let sys = ActorSystem::new(4);

    log.info("When workers are assigned for actor IDs 0, 1, and 4");
    let w0 = sys.assign_worker(0);
    let w1 = sys.assign_worker(1);
    let w4 = sys.assign_worker(4);

    log.info("Then different IDs map to different workers and mod-cycling holds");
    assert_ne!(w0, w1, "different actors should map to different workers");
    assert_eq!(w0, w4, "actor_id mod worker_count should cycle");
}

proptest! {
    #[test]
    fn mailbox_pending_count_matches_sent(n in 1usize..10) {
        let log = log();
        log.info("Given an ActorMailbox with large capacity");
        let mut mb = ActorMailbox::<PingMsg>::new(0, 100);

        log.info("When n messages are sent");
        for i in 0..n {
            mb.send(PingMsg { value: i as u32 }).unwrap();
        }

        log.info("Then pending() equals n");
        prop_assert_eq!(mb.pending(), n);
    }

    #[test]
    fn actor_worker_assignment_is_deterministic(actor_id in 0u32..1000, workers in 1usize..16) {
        let log = log();
        log.info("Given an ActorSystem");
        let sys = ActorSystem::new(workers);

        log.info("When assign_worker is called twice with the same actor_id");
        let w1 = sys.assign_worker(actor_id);
        let w2 = sys.assign_worker(actor_id);

        log.info("Then both calls return the same worker within valid range");
        prop_assert_eq!(w1, w2, "same actor_id must always map to same worker");
        prop_assert!(w1 < workers, "worker index must be within range");
    }
}
