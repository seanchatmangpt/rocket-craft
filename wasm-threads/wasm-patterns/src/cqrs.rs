pub trait Command: 'static {
    type Error: std::fmt::Debug;
    fn command_name(&self) -> &'static str;
}

pub trait Query: 'static {
    type Result;
    fn query_name(&self) -> &'static str;
}

pub trait CommandHandler<C: Command> {
    fn handle(&mut self, cmd: C) -> Result<(), C::Error>;
}

pub trait QueryHandler<Q: Query> {
    fn handle(&self, query: &Q) -> Q::Result;
}

// Generic in-memory command bus
pub struct CommandBus {
    commands_dispatched: u64,
    commands_failed: u64,
}

impl CommandBus {
    pub fn new() -> Self { Self { commands_dispatched: 0, commands_failed: 0 } }

    pub fn dispatch<C: Command, H: CommandHandler<C>>(&mut self, handler: &mut H, cmd: C) -> Result<(), C::Error> {
        self.commands_dispatched += 1;
        handler.handle(cmd).map_err(|e| { self.commands_failed += 1; e })
    }

    pub fn commands_dispatched(&self) -> u64 { self.commands_dispatched }
    pub fn commands_failed(&self) -> u64 { self.commands_failed }
    pub fn success_rate(&self) -> f64 {
        if self.commands_dispatched == 0 { return 1.0; }
        (self.commands_dispatched - self.commands_failed) as f64 / self.commands_dispatched as f64
    }
}

impl Default for CommandBus {
    fn default() -> Self { Self::new() }
}

pub struct QueryBus {
    queries_handled: u64,
}

impl QueryBus {
    pub fn new() -> Self { Self { queries_handled: 0 } }

    pub fn ask<Q: Query, H: QueryHandler<Q>>(&mut self, handler: &H, query: &Q) -> Q::Result {
        self.queries_handled += 1;
        handler.handle(query)
    }

    pub fn queries_handled(&self) -> u64 { self.queries_handled }
}

impl Default for QueryBus {
    fn default() -> Self { Self::new() }
}

// Read model: optimized projection for queries
#[derive(Debug, Default, Clone)]
pub struct ReadModel<T: Clone + Default> {
    data: T,
    version: u64,
}

impl<T: Clone + Default> ReadModel<T> {
    pub fn new() -> Self { Self { data: T::default(), version: 0 } }
    pub fn update<F: FnOnce(&mut T)>(&mut self, f: F) { f(&mut self.data); self.version += 1; }
    pub fn read(&self) -> &T { &self.data }
    pub fn version(&self) -> u64 { self.version }
}
