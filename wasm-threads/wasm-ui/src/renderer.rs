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

    fn present(&mut self) {
        self.frame_count += 1;
    }

    fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

// ── WebGL2Renderer (WASM32 only) ─────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn draw_char_gl(gl: &web_sys::WebGl2RenderingContext, canvas_height: f32, cx: f32, cy: f32, ch: char) {
    let bitmap: [u8; 5] = match ch {
        '0' => [0x3e, 0x51, 0x49, 0x45, 0x3e],
        '1' => [0x00, 0x42, 0x7f, 0x40, 0x00],
        '2' => [0x42, 0x61, 0x51, 0x49, 0x46],
        '3' => [0x21, 0x41, 0x45, 0x4b, 0x31],
        '4' => [0x18, 0x14, 0x12, 0x7f, 0x10],
        '5' => [0x27, 0x45, 0x45, 0x45, 0x39],
        '6' => [0x3c, 0x4a, 0x49, 0x49, 0x30],
        '7' => [0x01, 0x71, 0x09, 0x05, 0x03],
        '8' => [0x36, 0x49, 0x49, 0x49, 0x36],
        '9' => [0x06, 0x49, 0x49, 0x29, 0x1e],
        'S' | 's' => [0x26, 0x49, 0x49, 0x49, 0x32],
        'c' => [0x38, 0x44, 0x44, 0x44, 0x28],
        'o' => [0x38, 0x44, 0x44, 0x44, 0x38],
        'r' => [0x7c, 0x08, 0x04, 0x04, 0x08],
        'e' => [0x38, 0x54, 0x54, 0x54, 0x18],
        ':' => [0x00, 0x36, 0x36, 0x00, 0x00],
        'E' => [0x7f, 0x49, 0x49, 0x49, 0x41],
        'n' => [0x7c, 0x08, 0x04, 0x04, 0x78],
        't' => [0x04, 0x3f, 0x44, 0x40, 0x20],
        'i' => [0x00, 0x44, 0x7d, 0x40, 0x00],
        'T' => [0x01, 0x01, 0x7f, 0x01, 0x01],
        'k' => [0x7f, 0x08, 0x14, 0x22, 0x41],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00],
        _ => [0x7f, 0x41, 0x41, 0x41, 0x7f],
    };
    
    let pixel_scale: i32 = 2;
    for col in 0..5 {
        let col_data = bitmap[col];
        for row in 0..7 {
            if (col_data & (1 << row)) != 0 {
                let px = cx + (col as f32 * pixel_scale as f32);
                let py = cy + (row as f32 * pixel_scale as f32);
                let bg_y = canvas_height - py - pixel_scale as f32;
                gl.scissor(px as i32, bg_y as i32, pixel_scale, pixel_scale);
                gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn draw_str_gl(gl: &web_sys::WebGl2RenderingContext, canvas_height: f32, mut cx: f32, cy: f32, text: &str) {
    gl.enable(web_sys::WebGl2RenderingContext::SCISSOR_TEST);
    for ch in text.chars() {
        draw_char_gl(gl, canvas_height, cx, cy, ch);
        cx += 14.0; // 5 columns * 2 pixel scale + 4 pixels spacing
    }
    gl.disable(web_sys::WebGl2RenderingContext::SCISSOR_TEST);
}

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

    fn draw_health_bar(&mut self, x: f32, y: f32, width: f32, height: f32, percentage: f32) {
        use wasm_bindgen::JsCast;
        let canvas_height = self.gl.canvas().and_then(|c| {
            c.dyn_into::<web_sys::HtmlCanvasElement>().ok()
        }).map(|c| c.height() as f32).unwrap_or(600.0);
        
        self.gl.enable(web_sys::WebGl2RenderingContext::SCISSOR_TEST);
        
        // Background (gray)
        let bg_y = canvas_height - y - height;
        self.gl.scissor(x as i32, bg_y as i32, width as i32, height as i32);
        self.gl.clear_color(0.266, 0.266, 0.266, 1.0);
        self.gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
        
        // Foreground
        let fg_width = (width * percentage) as i32;
        if fg_width > 0 {
            let (r, g, b) = if percentage > 0.5 {
                (0.0, 0.8, 0.266) // #00cc44
            } else if percentage > 0.25 {
                (1.0, 0.8, 0.0) // #ffcc00
            } else {
                (0.8, 0.133, 0.0) // #cc2200
            };
            self.gl.scissor(x as i32, bg_y as i32, fg_width, height as i32);
            self.gl.clear_color(r, g, b, 1.0);
            self.gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
        }
        
        self.gl.disable(web_sys::WebGl2RenderingContext::SCISSOR_TEST);
    }

    fn draw_score(&mut self, score: u64) {
        use wasm_bindgen::JsCast;
        let canvas_height = self.gl.canvas().and_then(|c| {
            c.dyn_into::<web_sys::HtmlCanvasElement>().ok()
        }).map(|c| c.height() as f32).unwrap_or(600.0);
        
        self.gl.clear_color(1.0, 1.0, 1.0, 1.0); // white
        draw_str_gl(&self.gl, canvas_height, 10.0, 24.0, &format!("Score: {}", score));
    }

    fn draw_entity_count(&mut self, count: usize) {
        use wasm_bindgen::JsCast;
        let canvas_height = self.gl.canvas().and_then(|c| {
            c.dyn_into::<web_sys::HtmlCanvasElement>().ok()
        }).map(|c| c.height() as f32).unwrap_or(600.0);
        
        self.gl.clear_color(0.67, 0.67, 1.0, 1.0); // #aaaaff
        draw_str_gl(&self.gl, canvas_height, 10.0, 44.0, &format!("Entities: {}", count));
    }

    fn draw_tick(&mut self, tick: u64) {
        use wasm_bindgen::JsCast;
        let canvas_height = self.gl.canvas().and_then(|c| {
            c.dyn_into::<web_sys::HtmlCanvasElement>().ok()
        }).map(|c| c.height() as f32).unwrap_or(600.0);
        
        self.gl.clear_color(0.53, 0.53, 0.53, 1.0); // #888888
        draw_str_gl(&self.gl, canvas_height, 10.0, 60.0, &format!("Tick: {}", tick));
    }

    fn present(&mut self) {
        self.frame_count += 1;
    }

    fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TestRenderer draw calls ───────────────────────────────────────────────

    #[test]
    fn new_renderer_has_no_calls() {
        let r = TestRenderer::new();
        assert_eq!(r.call_count(), 0);
        assert_eq!(r.frame_count(), 0);
    }

    #[test]
    fn clear_records_draw_call() {
        let mut r = TestRenderer::new();
        r.clear();
        assert_eq!(r.call_count(), 1);
        assert!(r.has_call(&DrawCall::Clear));
    }

    #[test]
    fn draw_health_bar_records_call() {
        let mut r = TestRenderer::new();
        r.draw_health_bar(10.0, 20.0, 100.0, 10.0, 0.75);
        assert!(r.has_call(&DrawCall::HealthBar {
            x: 10.0, y: 20.0, width: 100.0, height: 10.0, percentage: 0.75
        }));
    }

    #[test]
    fn draw_score_records_call() {
        let mut r = TestRenderer::new();
        r.draw_score(9999);
        assert!(r.has_call(&DrawCall::Score(9999)));
    }

    #[test]
    fn draw_entity_count_records_call() {
        let mut r = TestRenderer::new();
        r.draw_entity_count(42);
        assert!(r.has_call(&DrawCall::EntityCount(42)));
    }

    #[test]
    fn draw_tick_records_call() {
        let mut r = TestRenderer::new();
        r.draw_tick(100);
        assert!(r.has_call(&DrawCall::Tick(100)));
    }

    #[test]
    fn present_increments_frame_count() {
        let mut r = TestRenderer::new();
        r.present();
        assert_eq!(r.frame_count(), 1);
        r.present();
        assert_eq!(r.frame_count(), 2);
    }

    #[test]
    fn present_records_present_draw_call() {
        let mut r = TestRenderer::new();
        r.present();
        assert!(r.has_call(&DrawCall::Present));
    }

    #[test]
    fn clear_calls_removes_all() {
        let mut r = TestRenderer::new();
        r.clear();
        r.draw_score(1);
        r.clear_calls();
        assert_eq!(r.call_count(), 0);
        // frame_count is unaffected by clearing the call buffer
    }

    #[test]
    fn calls_of_type_filters_correctly() {
        let mut r = TestRenderer::new();
        r.clear();
        r.draw_score(5);
        r.clear();
        let clears = r.calls_of_type("Clear");
        assert_eq!(clears.len(), 2);
        let scores = r.calls_of_type("Score");
        assert_eq!(scores.len(), 1);
    }

    #[test]
    fn full_frame_sequence() {
        let mut r = TestRenderer::new();
        r.clear();
        r.draw_health_bar(0.0, 0.0, 200.0, 15.0, 0.5);
        r.draw_score(100);
        r.draw_entity_count(3);
        r.draw_tick(1);
        r.present();

        assert_eq!(r.frame_count(), 1);
        assert_eq!(r.call_count(), 6);
        assert!(r.calls_of_type("HealthBar").len() == 1);
    }
}
