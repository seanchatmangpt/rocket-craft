use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_patterns::{Command, CommandBus, CommandHandler, Query, QueryBus, QueryHandler};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _buffer) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[derive(Debug)]
struct DamageCommand {
    target_id: u32,
    amount: u32,
}

#[derive(Debug)]
struct CqrsError(String);

impl std::fmt::Display for CqrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Command for DamageCommand {
    type Error = CqrsError;
    fn command_name(&self) -> &'static str {
        "DamageCommand"
    }
}

struct HealthWriteModel {
    health: std::collections::HashMap<u32, u32>,
}

impl CommandHandler<DamageCommand> for HealthWriteModel {
    fn handle(&mut self, cmd: DamageCommand) -> Result<(), CqrsError> {
        let hp = self.health.entry(cmd.target_id).or_insert(100);
        *hp = hp.saturating_sub(cmd.amount);
        Ok(())
    }
}

struct HealthQuery {
    target_id: u32,
}

impl Query for HealthQuery {
    type Result = u32;
    fn query_name(&self) -> &'static str {
        "HealthQuery"
    }
}

impl QueryHandler<HealthQuery> for HealthWriteModel {
    fn handle(&self, query: &HealthQuery) -> u32 {
        *self.health.get(&query.target_id).unwrap_or(&100)
    }
}

fn fresh_model() -> HealthWriteModel {
    HealthWriteModel { health: std::collections::HashMap::new() }
}

#[test]
fn command_dispatch_updates_query_result() {
    let log = log();
    log.info("Given a write model and command/query buses");
    let mut bus = CommandBus::new();
    let mut qbus = QueryBus::new();
    let mut model = fresh_model();

    log.info("When a DamageCommand is dispatched");
    bus.dispatch(&mut model, DamageCommand { target_id: 1, amount: 30 }).unwrap();

    log.info("Then the query result reflects the damage applied");
    let hp = qbus.ask(&model, &HealthQuery { target_id: 1 });
    assert_eq!(hp, 70);
}

#[test]
fn query_result_changes_after_command_is_dispatched() {
    let log = log();
    log.info("Given a write model and query/command buses");
    let mut bus = CommandBus::new();
    let mut qbus = QueryBus::new();
    let mut model = fresh_model();

    log.info("When a query is issued before and after a command");
    let before = qbus.ask(&model, &HealthQuery { target_id: 1 });
    bus.dispatch(&mut model, DamageCommand { target_id: 1, amount: 50 }).unwrap();
    let after = qbus.ask(&model, &HealthQuery { target_id: 1 });

    log.info("Then the query result must differ from the pre-command value");
    assert_ne!(before, after, "query must reflect state changes from commands");
}

#[test]
fn command_bus_tracks_dispatch_counts_and_success_rate() {
    let log = log();
    log.info("Given a CommandBus and write model");
    let mut bus = CommandBus::new();
    let mut model = fresh_model();

    log.info("When 5 commands are dispatched successfully");
    for _ in 0..5 {
        bus.dispatch(&mut model, DamageCommand { target_id: 1, amount: 1 }).unwrap();
    }

    log.info("Then commands_dispatched is 5, commands_failed is 0, success_rate is 1.0");
    assert_eq!(bus.commands_dispatched(), 5);
    assert_eq!(bus.commands_failed(), 0);
    assert_eq!(bus.success_rate(), 1.0);
}
