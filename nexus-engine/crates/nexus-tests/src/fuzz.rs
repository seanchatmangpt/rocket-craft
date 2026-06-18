use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Deterministic fuzzer seeded with a u64 — produces reproducible sequences
pub struct DeterministicFuzzer {
    rng: ChaCha8Rng,
}

impl DeterministicFuzzer {
    pub fn new(seed: u64) -> Self {
        DeterministicFuzzer {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.rng.next_u32() as f32) / (u32::MAX as f32)
    }

    /// Generate a random combat sequence: list of (action_type, direction) pairs
    /// action_type: 0=Attack, 1=Parry, 2=PerfectParry, 3=Dodge
    pub fn combat_sequence(&mut self, length: usize) -> Vec<(u8, u8)> {
        (0..length)
            .map(|_| {
                (self.next_u32() as u8 % 4, self.next_u32() as u8 % 3)
            })
            .collect()
    }

    /// Generate a random economic sequence: list of (from_player, to_player, amount) transfers
    pub fn transfer_sequence(
        &mut self,
        num_players: u64,
        length: usize,
    ) -> Vec<(u64, u64, u32)> {
        (0..length)
            .map(|_| {
                let from = self.next_u64() % num_players;
                let mut to = self.next_u64() % num_players;
                while to == from {
                    to = self.next_u64() % num_players;
                }
                let amount = (self.next_u32() % 1000) + 1;
                (from, to, amount)
            })
            .collect()
    }

    fn next_u64(&mut self) -> u64 {
        let hi = self.next_u32() as u64;
        let lo = self.next_u32() as u64;
        (hi << 32) | lo
    }
}

/// Known-bad fuzz corpus: inputs that historically caused bugs
pub struct KnownBadCorpus;

impl KnownBadCorpus {
    pub fn combat_edge_cases() -> Vec<(f32, f32, f32, f32)> {
        vec![
            // (base_damage, combo_mult, equipment_bonus, armor)
            (0.0, 1.0, 0.0, 0.0),                        // zero damage
            (f32::MIN_POSITIVE, 3.0, 100.0, 9999.0),     // tiny damage vs massive armor
            (10_000.0, 3.0, 100.0, 0.0),                 // maximum damage
            (1.0, 0.0, 0.0, 0.0),                        // zero multiplier edge
            (100.0, 3.0, 0.0, 99.0),                     // damage barely above floor after armor
        ]
    }

    pub fn gold_edge_cases() -> Vec<(u32, u32)> {
        vec![
            (0, 0),
            (0, 1),
            (1, 0),
            (u32::MAX, 0),
            (u32::MAX / 2, u32::MAX / 2),
        ]
    }
}
