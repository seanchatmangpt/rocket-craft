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
        UiState { _phase: PhantomData, ..self }
    }
}

impl Default for UiState<Unloaded> {
    fn default() -> Self {
        Self::new()
    }
}

impl UiState<Loading> {
    pub fn ready(self) -> UiState<Ready> {
        UiState { _phase: PhantomData, ..self }
    }

    pub fn fail(self, _reason: &str) -> UiState<Error> {
        UiState { _phase: PhantomData, ..self }
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
        UiState { _phase: PhantomData, ..self }
    }
}
