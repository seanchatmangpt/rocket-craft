use std::marker::PhantomData;

pub struct Unloaded;
pub struct Loading;
pub struct Ready;
pub struct Error;

pub struct UiState<S> {
    pub frame: u64,
    pub last_game_tick: u64,
    pub player_health: u32,
    pub player_health_max: u32,
    pub player_score: u64,
    pub entity_count: usize,
    pub messages_received: u64,
    _phase: PhantomData<S>,
}

impl UiState<Unloaded> {
    pub fn new() -> Self {
        Self {
            frame: 0,
            last_game_tick: 0,
            player_health: 0,
            player_health_max: 100,
            player_score: 0,
            entity_count: 0,
            messages_received: 0,
            _phase: PhantomData,
        }
    }

    pub fn start_loading(self) -> UiState<Loading> {
        UiState {
            frame: self.frame,
            last_game_tick: self.last_game_tick,
            player_health: self.player_health,
            player_health_max: self.player_health_max,
            player_score: self.player_score,
            entity_count: self.entity_count,
            messages_received: self.messages_received,
            _phase: PhantomData,
        }
    }
}

impl Default for UiState<Unloaded> {
    fn default() -> Self {
        Self::new()
    }
}

impl UiState<Loading> {
    pub fn ready(self) -> UiState<Ready> {
        UiState {
            frame: self.frame,
            last_game_tick: self.last_game_tick,
            player_health: self.player_health,
            player_health_max: self.player_health_max,
            player_score: self.player_score,
            entity_count: self.entity_count,
            messages_received: self.messages_received,
            _phase: PhantomData,
        }
    }

    pub fn fail(self, _reason: &str) -> UiState<Error> {
        UiState {
            frame: self.frame,
            last_game_tick: self.last_game_tick,
            player_health: self.player_health,
            player_health_max: self.player_health_max,
            player_score: self.player_score,
            entity_count: self.entity_count,
            messages_received: self.messages_received,
            _phase: PhantomData,
        }
    }
}

impl UiState<Ready> {
    pub fn render_frame(&mut self) {
        self.frame += 1;
    }

    pub fn on_message_received(&mut self) {
        self.messages_received += 1;
    }

    pub fn update_from_game(
        &mut self,
        tick: u64,
        hp: u32,
        hp_max: u32,
        score: u64,
        entities: usize,
    ) {
        self.last_game_tick = tick;
        self.player_health = hp;
        self.player_health_max = hp_max;
        self.player_score = score;
        self.entity_count = entities;
        self.on_message_received();
    }

    pub fn health_percentage(&self) -> f32 {
        if self.player_health_max == 0 {
            return 0.0;
        }
        self.player_health as f32 / self.player_health_max as f32
    }
}

impl UiState<Error> {
    pub fn retry(self) -> UiState<Unloaded> {
        UiState {
            frame: self.frame,
            last_game_tick: self.last_game_tick,
            player_health: self.player_health,
            player_health_max: self.player_health_max,
            player_score: self.player_score,
            entity_count: self.entity_count,
            messages_received: self.messages_received,
            _phase: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ui_state_starts_unloaded() {
        let s = UiState::<Unloaded>::new();
        assert_eq!(s.frame, 0);
        assert_eq!(s.player_health_max, 100);
    }

    #[test]
    fn start_loading_transitions() {
        let loading = UiState::<Unloaded>::new().start_loading();
        // Loading state is accessible; its fields carry over
        assert_eq!(loading.player_health_max, 100);
    }

    #[test]
    fn loading_ready_transition() {
        let ready = UiState::<Unloaded>::new().start_loading().ready();
        assert_eq!(ready.frame, 0);
    }

    #[test]
    fn loading_fail_transition() {
        let err = UiState::<Unloaded>::new().start_loading().fail("io error");
        // Error state carries messages_received
        assert_eq!(err.messages_received, 0);
    }

    #[test]
    fn render_frame_increments_frame() {
        let mut r = UiState::<Unloaded>::new().start_loading().ready();
        r.render_frame();
        r.render_frame();
        assert_eq!(r.frame, 2);
    }

    #[test]
    fn update_from_game_sets_fields_and_counts_message() {
        let mut r = UiState::<Unloaded>::new().start_loading().ready();
        r.update_from_game(5, 80, 100, 1000, 3);
        assert_eq!(r.last_game_tick, 5);
        assert_eq!(r.player_health, 80);
        assert_eq!(r.player_score, 1000);
        assert_eq!(r.entity_count, 3);
        assert_eq!(r.messages_received, 1);
    }

    #[test]
    fn health_percentage_is_correct() {
        let mut r = UiState::<Unloaded>::new().start_loading().ready();
        r.update_from_game(0, 50, 100, 0, 0);
        let pct = r.health_percentage();
        assert!((pct - 0.5).abs() < 1e-6);
    }

    #[test]
    fn health_percentage_zero_max_is_zero() {
        let mut r = UiState::<Unloaded>::new().start_loading().ready();
        r.update_from_game(0, 0, 0, 0, 0);
        assert_eq!(r.health_percentage(), 0.0);
    }

    #[test]
    fn error_retry_returns_unloaded() {
        let restarted = UiState::<Unloaded>::new()
            .start_loading()
            .fail("bad")
            .retry();
        assert_eq!(restarted.frame, 0);
    }
}
