use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_core::WorkerChannel;

fn make_logger() -> Logger {
    let mut logger = Logger::new();
    let (sink, _buffer) = TuiBufferSink::new();
    logger.add_sink(Box::new(sink));
    logger
}

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
struct Ping {
    id: u32,
}

#[test]
fn channel_send_receive_typed_messages() {
    let mut log = make_logger();
    log.info("Given a WorkerChannel<Ping> with worker_id 0");
    let mut ch = WorkerChannel::<Ping>::new(0);

    log.info("When we send two Ping messages");
    ch.send(Ping { id: 42 }).unwrap();
    ch.send(Ping { id: 99 }).unwrap();

    log.info("Then pending_count is 2 and first receive yields Ping { id: 42 }");
    assert_eq!(ch.pending_count(), 2);
    assert_eq!(ch.receive(), Some(Ping { id: 42 }));
    assert_eq!(ch.pending_count(), 1);
}

#[test]
fn channel_receive_empty_returns_none() {
    let mut log = make_logger();
    log.info("Given a WorkerChannel<u32> with no messages");
    let mut ch = WorkerChannel::<u32>::new(0);

    log.info("When we call receive()");
    log.info("Then it returns None");
    assert_eq!(ch.receive(), None);
}

#[test]
fn channel_drain_clears_queue() {
    let mut log = make_logger();
    log.info("Given a WorkerChannel<u32> with three messages enqueued");
    let mut ch = WorkerChannel::<u32>::new(0);
    ch.send(1).unwrap();
    ch.send(2).unwrap();
    ch.send(3).unwrap();

    log.info("When we call drain()");
    let drained = ch.drain();

    log.info("Then all messages are returned in order and the queue is empty");
    assert_eq!(drained, vec![1, 2, 3]);
    assert_eq!(ch.pending_count(), 0);
}
