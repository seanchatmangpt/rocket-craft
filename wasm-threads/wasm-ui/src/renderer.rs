/// Renderer trait — native impl uses a Vec buffer, WASM impl uses web-sys Canvas
pub trait Renderer {
    fn clear(&mut self);
    fn draw_health_bar(&mut self, x: f32, y: f32, width: f32, height: f32, percentage: f32);
    fn draw_score(&mut self, score: u64);
    fn draw_entity_count(&mut self, count: usize);
    fn draw_tick(&mut self, tick: u64);
    fn present(&mut self);
    fn frame_count(&self) -> u64;
}

/// Native test renderer — records draw calls for verification
#[derive(Debug, Default)]
pub struct TestRenderer {
    frame_count: u64,
    draw_calls: Vec<DrawCall>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawCall {
    Clear,
    HealthBar {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        percentage: f32,
    },
    Score(u64),
    EntityCount(usize),
    Tick(u64),
    Present,
}

impl Renderer for TestRenderer {
    fn clear(&mut self) {
        self.draw_calls.push(DrawCall::Clear);
    }

    fn draw_health_bar(&mut self, x: f32, y: f32, width: f32, height: f32, percentage: f32) {
        self.draw_calls.push(DrawCall::HealthBar {
            x,
            y,
            width,
            height,
            percentage,
        });
    }

    fn draw_score(&mut self, score: u64) {
        self.draw_calls.push(DrawCall::Score(score));
    }

    fn draw_entity_count(&mut self, count: usize) {
        self.draw_calls.push(DrawCall::EntityCount(count));
    }

    fn draw_tick(&mut self, tick: u64) {
        self.draw_calls.push(DrawCall::Tick(tick));
    }

    fn present(&mut self) {
        self.draw_calls.push(DrawCall::Present);
        self.frame_count += 1;
    }

    fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl TestRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw_calls(&self) -> &[DrawCall] {
        &self.draw_calls
    }

    pub fn clear_calls(&mut self) {
        self.draw_calls.clear();
    }

    pub fn call_count(&self) -> usize {
        self.draw_calls.len()
    }

    pub fn has_call(&self, call: &DrawCall) -> bool {
        self.draw_calls.contains(call)
    }

    pub fn calls_of_type(&self, variant: &str) -> Vec<&DrawCall> {
        self.draw_calls
            .iter()
            .filter(|c| match (c, variant) {
                (DrawCall::Clear, "Clear") => true,
                (DrawCall::HealthBar { .. }, "HealthBar") => true,
                (DrawCall::Score(_), "Score") => true,
                (DrawCall::Present, "Present") => true,
                _ => false,
            })
            .collect()
    }
}
