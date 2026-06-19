use std::collections::VecDeque;

pub trait ActorMessage: Send + 'static {}
pub trait ActorHandler<M: ActorMessage> {
    type State;
    fn handle(&mut self, state: &mut Self::State, msg: M);
    fn on_start(&mut self, _state: &mut Self::State) {
        let _ = _state;
    }
    fn on_stop(&mut self, _state: &mut Self::State) {
        let _ = _state;
    }
}

pub struct ActorMailbox<M: ActorMessage> {
    queue: VecDeque<M>,
    actor_id: u32,
    capacity: usize,
}

impl<M: ActorMessage> ActorMailbox<M> {
    pub fn new(actor_id: u32, capacity: usize) -> Self {
        Self { queue: VecDeque::with_capacity(capacity), actor_id, capacity }
    }
    pub fn send(&mut self, msg: M) -> Result<(), ActorError> {
        if self.queue.len() >= self.capacity {
            return Err(ActorError::MailboxFull { actor_id: self.actor_id, capacity: self.capacity });
        }
        self.queue.push_back(msg);
        Ok(())
    }
    pub fn receive(&mut self) -> Option<M> { self.queue.pop_front() }
    pub fn pending(&self) -> usize { self.queue.len() }
    pub fn actor_id(&self) -> u32 { self.actor_id }
    pub fn is_full(&self) -> bool { self.queue.len() >= self.capacity }
}

#[derive(Debug)]
pub enum ActorError {
    MailboxFull { actor_id: u32, capacity: usize },
    ActorNotFound(u32),
}
impl std::fmt::Display for ActorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MailboxFull { actor_id, capacity } => write!(f, "Actor {actor_id} mailbox full (cap={capacity})"),
            Self::ActorNotFound(id) => write!(f, "Actor {id} not found"),
        }
    }
}
impl std::error::Error for ActorError {}

pub struct ActorSystem {
    next_id: u32,
    worker_count: usize,
}

impl ActorSystem {
    pub fn new(worker_count: usize) -> Self { Self { next_id: 0, worker_count } }
    pub fn spawn_actor_id(&mut self) -> u32 { let id = self.next_id; self.next_id += 1; id }
    pub fn worker_count(&self) -> usize { self.worker_count }
    pub fn actor_count(&self) -> u32 { self.next_id }
    pub fn assign_worker(&self, actor_id: u32) -> usize { (actor_id as usize) % self.worker_count }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Ping;
    impl ActorMessage for Ping {}

    // ── ActorMailbox ──────────────────────────────────────────────────────────

    #[test]
    fn new_mailbox_is_empty() {
        let mb: ActorMailbox<Ping> = ActorMailbox::new(7, 10);
        assert_eq!(mb.pending(), 0);
        assert_eq!(mb.actor_id(), 7);
        assert!(!mb.is_full());
    }

    #[test]
    fn send_enqueues_message() {
        let mut mb: ActorMailbox<Ping> = ActorMailbox::new(0, 10);
        mb.send(Ping).unwrap();
        assert_eq!(mb.pending(), 1);
    }

    #[test]
    fn receive_dequeues_fifo() {
        #[derive(Debug)]
        struct Msg(u32);
        impl ActorMessage for Msg {}

        let mut mb: ActorMailbox<Msg> = ActorMailbox::new(0, 10);
        mb.send(Msg(1)).unwrap();
        mb.send(Msg(2)).unwrap();
        assert_eq!(mb.receive().unwrap().0, 1);
        assert_eq!(mb.receive().unwrap().0, 2);
        assert!(mb.receive().is_none());
    }

    #[test]
    fn send_when_full_returns_err() {
        let mut mb: ActorMailbox<Ping> = ActorMailbox::new(0, 2);
        mb.send(Ping).unwrap();
        mb.send(Ping).unwrap();
        assert!(mb.is_full());
        assert!(mb.send(Ping).is_err());
    }

    // ── ActorSystem ───────────────────────────────────────────────────────────

    #[test]
    fn spawn_actor_id_is_sequential() {
        let mut sys = ActorSystem::new(4);
        assert_eq!(sys.spawn_actor_id(), 0);
        assert_eq!(sys.spawn_actor_id(), 1);
        assert_eq!(sys.actor_count(), 2);
    }

    #[test]
    fn assign_worker_distributes_round_robin() {
        let sys = ActorSystem::new(4);
        assert_eq!(sys.assign_worker(0), 0);
        assert_eq!(sys.assign_worker(1), 1);
        assert_eq!(sys.assign_worker(4), 0); // wraps
    }

    #[test]
    fn worker_count_matches_config() {
        let sys = ActorSystem::new(8);
        assert_eq!(sys.worker_count(), 8);
    }
}
