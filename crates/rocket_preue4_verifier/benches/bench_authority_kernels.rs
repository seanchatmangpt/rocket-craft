use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rocket_preue4_verifier::{
    authority::AuthorityState,
    simd::batch_update_damage_simd_equiv,
    transitions::{TransitionTable, batch_update_damage_scalar, batch_update_damage_table},
};

fn bench_scalar(c: &mut Criterion) {
    let mut state = AuthorityState::new(100_000);
    for i in 0..100_000 {
        state.heat[i] = (i % 16) as u8;
        state.stress[i] = (i % 12) as u8;
        state.socket_health[i] = (15 - i % 16) as u8;
    }
    c.bench_function("batch_update_damage_scalar_100k", |b| {
        b.iter(|| batch_update_damage_scalar(black_box(&mut state.clone())))
    });
}

fn bench_table(c: &mut Criterion) {
    let mut state = AuthorityState::new(100_000);
    for i in 0..100_000 {
        state.heat[i] = (i % 16) as u8;
        state.stress[i] = (i % 12) as u8;
        state.socket_health[i] = (15 - i % 16) as u8;
    }
    let table = TransitionTable::build();
    c.bench_function("batch_update_damage_table_100k", |b| {
        b.iter(|| batch_update_damage_table(black_box(&mut state.clone()), &table))
    });
}

fn bench_simd_equiv(c: &mut Criterion) {
    let heat: Vec<u8> = (0..100_000).map(|i| (i % 16) as u8).collect();
    let stress: Vec<u8> = (0..100_000).map(|i| (i % 12) as u8).collect();
    let socket: Vec<u8> = (0..100_000).map(|i| (15 - i % 16) as u8).collect();
    let mut damage = vec![0u8; 100_000];
    c.bench_function("batch_update_damage_simd_equiv_100k", |b| {
        b.iter(|| {
            batch_update_damage_simd_equiv(
                black_box(&heat),
                black_box(&stress),
                black_box(&socket),
                black_box(&mut damage),
            )
        })
    });
}

criterion_group!(benches, bench_scalar, bench_table, bench_simd_equiv);
criterion_main!(benches);
