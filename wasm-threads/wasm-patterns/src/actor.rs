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
