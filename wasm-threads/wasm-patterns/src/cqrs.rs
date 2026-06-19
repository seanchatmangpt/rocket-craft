pub trait Command: 'static {
    type Error: std::fmt::Debug;
    fn command_name(&self) -> &'static str;
}

#[derive(Debug)]
pub struct StringCommand(pub String);

impl Command for StringCommand {
    type Error = &'static str;
    fn command_name(&self) -> &'static str {
        "string_command"
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── Test fixture: a simple command handler ────────────────────────────────

    struct AddHandler { sum: i64 }

    #[derive(Debug)]
    struct AddCmd(i64);
    impl Command for AddCmd {
        type Error = &'static str;
        fn command_name(&self) -> &'static str { "add" }
    }

    impl CommandHandler<AddCmd> for AddHandler {
        fn handle(&mut self, cmd: AddCmd) -> Result<(), &'static str> {
            self.sum += cmd.0;
            Ok(())
        }
    }

    struct AlwaysFailHandler;
    impl CommandHandler<AddCmd> for AlwaysFailHandler {
        fn handle(&mut self, _cmd: AddCmd) -> Result<(), &'static str> {
            Err("forced failure")
        }
    }

    // ── Test fixture: a simple query ──────────────────────────────────────────

    struct SumQuery;
    impl Query for SumQuery {
        type Result = i64;
        fn query_name(&self) -> &'static str { "sum" }
    }

    struct SumQueryHandler(i64);
    impl QueryHandler<SumQuery> for SumQueryHandler {
        fn handle(&self, _q: &SumQuery) -> i64 { self.0 }
    }

    // ── CommandBus ────────────────────────────────────────────────────────────

    #[test]
    fn new_bus_has_zero_counts() {
        let bus = CommandBus::new();
        assert_eq!(bus.commands_dispatched(), 0);
        assert_eq!(bus.commands_failed(), 0);
    }

    #[test]
    fn dispatch_success_increments_dispatched() {
        let mut bus = CommandBus::new();
        let mut h = AddHandler { sum: 0 };
        bus.dispatch(&mut h, AddCmd(5)).unwrap();
        assert_eq!(bus.commands_dispatched(), 1);
        assert_eq!(bus.commands_failed(), 0);
        assert_eq!(h.sum, 5);
    }

    #[test]
    fn dispatch_failure_increments_both_counters() {
        let mut bus = CommandBus::new();
        let mut h = AlwaysFailHandler;
        let _ = bus.dispatch(&mut h, AddCmd(1));
        assert_eq!(bus.commands_dispatched(), 1);
        assert_eq!(bus.commands_failed(), 1);
    }

    #[test]
    fn success_rate_one_when_no_dispatches() {
        let bus = CommandBus::new();
        assert_eq!(bus.success_rate(), 1.0);
    }

    #[test]
    fn success_rate_half_when_one_of_two_fails() {
        let mut bus = CommandBus::new();
        let mut ok = AddHandler { sum: 0 };
        let mut fail = AlwaysFailHandler;
        bus.dispatch(&mut ok, AddCmd(1)).unwrap();
        let _ = bus.dispatch(&mut fail, AddCmd(1));
        assert!((bus.success_rate() - 0.5).abs() < 1e-9);
    }

    // ── QueryBus ─────────────────────────────────────────────────────────────

    #[test]
    fn ask_delegates_to_handler() {
        let mut bus = QueryBus::new();
        let h = SumQueryHandler(42);
        let result = bus.ask(&h, &SumQuery);
        assert_eq!(result, 42);
        assert_eq!(bus.queries_handled(), 1);
    }

    // ── ReadModel ─────────────────────────────────────────────────────────────

    #[test]
    fn read_model_starts_at_version_zero() {
        let model: ReadModel<i64> = ReadModel::new();
        assert_eq!(model.version(), 0);
        assert_eq!(*model.read(), 0);
    }

    #[test]
    fn update_increments_version() {
        let mut model: ReadModel<i64> = ReadModel::new();
        model.update(|d| *d += 10);
        model.update(|d| *d += 5);
        assert_eq!(model.version(), 2);
        assert_eq!(*model.read(), 15);
    }
}
