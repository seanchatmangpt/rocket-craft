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

// ── CanvasRenderer (WASM32 only) ─────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub struct CanvasRenderer {
    ctx: web_sys::CanvasRenderingContext2d,
    frame_count: u64,
}

#[cfg(target_arch = "wasm32")]
impl CanvasRenderer {
    /// Create from an existing canvas 2D context.
    pub fn new(ctx: web_sys::CanvasRenderingContext2d) -> Self {
        Self { ctx, frame_count: 0 }
    }
}

#[cfg(target_arch = "wasm32")]
impl Renderer for CanvasRenderer {
    fn clear(&mut self) {
        let canvas = self.ctx.canvas().unwrap();
        self.ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }

    fn draw_health_bar(&mut self, x: f32, y: f32, width: f32, height: f32, percentage: f32) {
        // Background (gray)
        self.ctx.set_fill_style_str("#444444");
        self.ctx.fill_rect(x as f64, y as f64, width as f64, height as f64);
        // Foreground (green/yellow/red based on percentage)
        let color = if percentage > 0.5 {
            "#00cc44"
        } else if percentage > 0.25 {
            "#ffcc00"
        } else {
            "#cc2200"
        };
        self.ctx.set_fill_style_str(color);
        self.ctx.fill_rect(x as f64, y as f64, (width * percentage) as f64, height as f64);
    }

    fn draw_score(&mut self, score: u64) {
        self.ctx.set_fill_style_str("#ffffff");
        self.ctx.set_font("16px monospace");
        let _ = self.ctx.fill_text(&format!("Score: {}", score), 10.0, 24.0);
    }

    fn draw_entity_count(&mut self, count: usize) {
        self.ctx.set_fill_style_str("#aaaaff");
        self.ctx.set_font("14px monospace");
        let _ = self.ctx.fill_text(&format!("Entities: {}", count), 10.0, 44.0);
    }

    fn draw_tick(&mut self, tick: u64) {
        self.ctx.set_fill_style_str("#888888");
        self.ctx.set_font("12px monospace");
        let _ = self.ctx.fill_text(&format!("Tick: {}", tick), 10.0, 60.0);
    }

    fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

// ── WebGL2Renderer (WASM32 only) ─────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub struct WebGL2Renderer {
    gl: web_sys::WebGl2RenderingContext,
    frame_count: u64,
}

#[cfg(target_arch = "wasm32")]
impl WebGL2Renderer {
    pub fn new(gl: web_sys::WebGl2RenderingContext) -> Self {
        Self { gl, frame_count: 0 }
    }

    pub fn gl(&self) -> &web_sys::WebGl2RenderingContext {
        &self.gl
    }
}

#[cfg(target_arch = "wasm32")]
impl Renderer for WebGL2Renderer {
    fn clear(&mut self) {
        self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
    }

    fn draw_health_bar(&mut self, _x: f32, _y: f32, _width: f32, _height: f32, _percentage: f32) {
        // Real WebGL 2.0 draw calls would go here
    }

    fn draw_score(&mut self, _score: u64) {
        // Text rendering in WebGL 2.0 requires a texture or separate overlay
    }

    fn draw_entity_count(&mut self, _count: usize) {
    }

    fn draw_tick(&mut self, _tick: u64) {
    }

    fn present(&mut self) {
        self.frame_count += 1;
    }

    fn frame_count(&self) -> u64 {
        self.frame_count
    }
}
