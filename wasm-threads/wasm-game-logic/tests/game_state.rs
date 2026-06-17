use chicago_tdd_tools::{Logger, TuiBufferSink};
use wasm_game_logic::{GameState, Health, Initializing, Player, World};

fn log() -> Logger {
    let mut l = Logger::new();
    let (sink, _) = TuiBufferSink::new();
    l.add_sink(Box::new(sink));
    l
}

#[test]
fn game_state_start_transitions_to_running() {
    let mut log = log();
    log.info("Given a GameState in the Initializing phase");
    let state = GameState::<Initializing>::new();

    log.info("When start() is called");
    let running = state.start();

    log.info("Then is_running() returns true");
    assert!(running.is_running());
}

#[test]
fn game_state_tick_increments_on_each_call() {
    let mut log = log();
    log.info("Given a Running GameState with tick=0");
    let mut state = GameState::<Initializing>::new().start();
    assert_eq!(state.tick, 0);

    log.info("When tick(16) is called once");
    state.tick(16);
    log.info("Then tick counter is 1");
    assert_eq!(state.tick, 1);

    log.info("When tick(16) is called again");
    state.tick(16);
    log.info("Then tick counter is 2");
    assert_eq!(state.tick, 2);
}

#[test]
fn game_state_elapsed_ms_accumulates() {
    let mut log = log();
    log.info("Given a Running GameState");
    let mut state = GameState::<Initializing>::new().start();

    log.info("When we tick with 100 ms then 200 ms");
    state.tick(100);
    state.tick(200);

    log.info("Then elapsed_ms is 300");
    assert_eq!(state.elapsed_ms, 300);
}

#[test]
fn game_state_pause_and_resume_cycle() {
    let mut log = log();
    log.info("Given a Running GameState");
    let state = GameState::<Initializing>::new().start();

    log.info("When pause() is called");
    let paused = state.pause();
    log.info("Then is_paused() returns true");
    assert!(paused.is_paused());

    log.info("When resume() is called");
    let running = paused.resume();
    log.info("Then is_running() returns true again");
    assert!(running.is_running());
}

#[test]
fn game_state_running_to_game_over() {
    let mut log = log();
    log.info("Given a Running GameState with no ticks");
    let state = GameState::<Initializing>::new().start();

    log.info("When game_over() is called");
    let over = state.game_over();

    log.info("Then total_ticks() returns 0");
    assert_eq!(over.total_ticks(), 0);
}

#[test]
fn game_state_game_over_restart_produces_fresh_state() {
    let mut log = log();
    log.info("Given a GameOver state");
    let over = GameState::<Initializing>::new().start().game_over();

    log.info("When restart() is called");
    let fresh = over.restart();

    log.info("Then starting the fresh state gives tick=0");
    let running = fresh.start();
    assert_eq!(running.tick, 0);
}

#[test]
fn game_over_winner_score_from_surviving_players() {
    let mut log = log();
    log.info("Given a Running GameState with one player entity having score 9999");
    let state = GameState::<Initializing>::new();
    let mut running = state.start();
    let e = running.world.spawn();
    running.world.add_health(e, Health::new(100));
    running.world.add_player(
        e,
        Player {
            name: "Hero".into(),
            score: 9999,
        },
    );

    log.info("When the game transitions to GameOver");
    let over = running.game_over();

    log.info("Then winner_score() returns 9999");
    assert_eq!(over.winner_score(), 9999);
}

#[test]
fn game_over_winner_score_is_zero_when_no_players() {
    let mut log = log();
    log.info("Given a Running GameState with no player entities");
    let state = GameState::<Initializing>::new().start();

    log.info("When game transitions to GameOver");
    let over = state.game_over();

    log.info("Then winner_score() returns 0");
    assert_eq!(over.winner_score(), 0);
}

#[test]
fn game_state_total_ticks_reflects_run_time() {
    let mut log = log();
    log.info("Given a Running GameState");
    let mut state = GameState::<Initializing>::new().start();

    log.info("When we tick 5 times");
    for _ in 0..5 {
        state.tick(16);
    }

    log.info("And transition to GameOver");
    let over = state.game_over();

    log.info("Then total_ticks() is 5");
    assert_eq!(over.total_ticks(), 5);
}
