// GC-MECHBIRTH-002: Stress tests for authority field scalar/table kernels.
// These tests prove that batch_update_damage_scalar, batch_update_damage_table,
// and the SIMD-equivalence checker all pass at 10k and 100k scale without panic or divergence.

use rocket_preue4_verifier::{
    authority::AuthorityState,
    simd::verify_simd_scalar_equivalence,
    transitions::{TransitionTable, batch_update_damage_scalar, batch_update_damage_table},
};
use std::time::Instant;

#[test]
fn stress_10k_authority_cells() {
    let n = 10_000;
    let mut state = AuthorityState::new(n);
    for i in 0..n {
        state.heat[i] = (i % 16) as u8;
        state.stress[i] = (i % 12) as u8;
        state.socket_health[i] = (15 - i % 16) as u8;
    }
    let t0 = Instant::now();
    batch_update_damage_scalar(&mut state);
    let elapsed = t0.elapsed();
    println!("[stress] 10k scalar: {:?}", elapsed);
    assert!(state.damage.iter().all(|&d| d <= 15));
}

#[test]
fn stress_100k_authority_cells() {
    let n = 100_000;
    let mut state = AuthorityState::new(n);
    for i in 0..n {
        state.heat[i] = (i % 16) as u8;
        state.stress[i] = (i % 12) as u8;
        state.socket_health[i] = (15 - i % 16) as u8;
    }
    let t0 = Instant::now();
    batch_update_damage_scalar(&mut state);
    let elapsed_scalar = t0.elapsed();

    let table = TransitionTable::build();
    let t1 = Instant::now();
    batch_update_damage_table(&mut state, &table);
    let elapsed_table = t1.elapsed();

    println!(
        "[stress] 100k scalar: {:?}, table: {:?}",
        elapsed_scalar, elapsed_table
    );
    assert!(state.damage.iter().all(|&d| d <= 15));
}

#[test]
fn stress_100k_simd_equiv() {
    let n = 100_000;
    let heat: Vec<u8> = (0..n).map(|i| (i % 16) as u8).collect();
    let stress: Vec<u8> = (0..n).map(|i| (i % 12) as u8).collect();
    let socket: Vec<u8> = (0..n).map(|i| (15 - i % 16) as u8).collect();
    let t0 = Instant::now();
    let result = verify_simd_scalar_equivalence(&heat, &stress, &socket);
    let elapsed = t0.elapsed();
    println!("[stress] 100k simd_equiv: {:?}", elapsed);
    assert!(result.is_ok(), "SIMD divergence: {:?}", result.err());
}
