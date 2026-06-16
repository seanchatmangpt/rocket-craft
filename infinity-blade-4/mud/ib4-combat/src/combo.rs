pub struct ComboTracker {
    pub depth: u32,
    turns_since_attack: u32,
    reset_threshold: u32,
}

impl ComboTracker {
    pub fn new(reset_threshold: u32) -> Self {
        ComboTracker {
            depth: 0,
            turns_since_attack: 0,
            reset_threshold,
        }
    }

    /// Call when player successfully attacks.
    pub fn on_hit(&mut self) {
        self.depth += 1;
        self.turns_since_attack = 0;
    }

    /// Call when player does anything other than attack.
    pub fn on_non_attack_turn(&mut self) {
        self.turns_since_attack += 1;
        if self.turns_since_attack >= self.reset_threshold {
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        self.depth = 0;
        self.turns_since_attack = 0;
    }

    pub fn multiplier(&self) -> f32 {
        match self.depth {
            0 | 1 => 1.0,
            2 => 1.5,
            3 => 2.0,
            _ => 3.0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.depth > 0
    }
}
