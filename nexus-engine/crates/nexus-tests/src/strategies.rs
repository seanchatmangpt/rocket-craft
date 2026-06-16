use proptest::prelude::*;

/// Strategy for a valid HP value (0.0 to 100_000.0)
pub fn hp_strategy() -> impl Strategy<Value = f32> {
    0.0f32..100_000.0f32
}

/// Strategy for gold amounts (u32)
pub fn gold_strategy() -> impl Strategy<Value = u32> {
    0u32..=1_000_000u32
}

/// Strategy for combo depth (0 to 7)
pub fn combo_depth_strategy() -> impl Strategy<Value = u32> {
    0u32..=7u32
}

/// Strategy for time dilation factor (0.35 to 1.3 as in game spec)
pub fn time_dilation_strategy() -> impl Strategy<Value = f32> {
    0.35f32..=1.3f32
}

/// Strategy for damage values (1.0 to 10_000.0)
pub fn damage_strategy() -> impl Strategy<Value = f32> {
    1.0f32..10_000.0f32
}

/// Strategy for armor values (0.0 to 500.0)
pub fn armor_strategy() -> impl Strategy<Value = f32> {
    0.0f32..500.0f32
}

/// Strategy for attack direction (0=Overhead, 1=Left, 2=Right)
pub fn attack_dir_strategy() -> impl Strategy<Value = u8> {
    0u8..3u8
}

/// Strategy for player bloodline (0 to 25)
pub fn bloodline_strategy() -> impl Strategy<Value = u32> {
    0u32..=25u32
}

/// Strategy for item price (1 to 100_000 gold)
pub fn price_strategy() -> impl Strategy<Value = u32> {
    1u32..=100_000u32
}

/// Strategy for XP amounts
pub fn xp_strategy() -> impl Strategy<Value = u64> {
    0u64..=1_000_000u64
}

/// Strategy for player rating (ELO range)
pub fn elo_rating_strategy() -> impl Strategy<Value = u32> {
    800u32..=3000u32
}

/// Strategy for a valid 3D position component (-10_000.0 to 10_000.0)
pub fn world_coord_strategy() -> impl Strategy<Value = f32> {
    -10_000.0f32..10_000.0f32
}
