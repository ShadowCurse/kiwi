#![allow(unused)]
use criterion::{criterion_group, criterion_main, Criterion};

mod bench_apecs;
mod bench_bevy;
mod bench_hecs;
mod bench_kiwi;
mod bench_legion;
mod bench_planck_ecs;
mod bench_shipyard;
mod bench_specs;

fn bench_simple_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_insert");

    group.bench_function("apecs", |b| {
        let mut bench = bench_apecs::simple_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("bevy", |b| {
        let mut bench = bench_bevy::simple_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("hecs", |b| {
        let mut bench = bench_hecs::simple_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("kiwi", |b| {
        let mut bench = bench_kiwi::simple_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("legion", |b| {
        let mut bench = bench_legion::simple_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("planck_ecs", |b| {
        let mut bench = bench_planck_ecs::simple_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("shipyard", |b| {
        let mut bench = bench_shipyard::simple_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("specs", |b| {
        let mut bench = bench_specs::simple_insert::Benchmark::new();
        b.iter(move || bench.run());
    });
}

fn bench_add_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_remove_component");

    group.bench_function("apecs", |b| {
        let mut bench = bench_apecs::add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("bevy", |b| {
        let mut bench = bench_bevy::add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("hecs", |b| {
        let mut bench = bench_hecs::add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("kiwi", |b| {
        let mut bench = bench_kiwi::add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("legion", |b| {
        let mut bench = bench_legion::add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("planck_ecs", |b| {
        let mut bench = bench_planck_ecs::add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("shipyard", |b| {
        let mut bench = bench_shipyard::add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("specs", |b| {
        let mut bench = bench_specs::add_remove::Benchmark::new();
        b.iter(move || bench.run());
    });

    group.finish();
}

fn bench_simple_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_iter");

    group.bench_function("apecs", |b| {
        let mut bench = bench_apecs::simple_iter::Benchmark::new().unwrap();
        b.iter(move || bench.run());
    });
    group.bench_function("bevy", |b| {
        let mut bench = bench_bevy::simple_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("hecs", |b| {
        let mut bench = bench_hecs::simple_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("kiwi", |b| {
        let mut bench = bench_kiwi::simple_iter::Benchmark::new();
        b.iter(|| bench.run());
    });
    group.bench_function("legion", |b| {
        let mut bench = bench_legion::simple_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("planck_ecs", |b| {
        let mut bench = bench_planck_ecs::simple_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("shipyard", |b| {
        let mut bench = bench_shipyard::simple_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("specs", |b| {
        let mut bench = bench_specs::simple_iter::Benchmark::new();
        b.iter(move || bench.run());
    });

    group.finish();
}

fn bench_frag_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("frag_iter");

    group.bench_function("apecs", |b| {
        let mut store = bench_apecs::frag_iter::arch();
        b.iter(move || bench_apecs::frag_iter::tick_arch(&mut store))
    });
    group.bench_function("bevy", |b| {
        let mut bench = bench_bevy::frag_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("hecs", |b| {
        let mut bench = bench_hecs::frag_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("kiwi", |b| {
        let mut bench = bench_kiwi::frag_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("legion", |b| {
        let mut bench = bench_legion::frag_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("planck_ecs", |b| {
        let mut bench = bench_planck_ecs::frag_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("shipyard", |b| {
        let mut bench = bench_shipyard::frag_iter::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("specs", |b| {
        let mut bench = bench_specs::frag_iter::Benchmark::new();
        b.iter(move || bench.run());
    });

    group.finish();
}

fn bench_schedule(c: &mut Criterion) {
    let mut group = c.benchmark_group("schedule");

    group.bench_function("apecs", |b| {
        let mut bench = bench_apecs::schedule::Benchmark::new();
        b.iter(move || bench.run())
    });
    group.bench_function("bevy", |b| {
        let mut bench = bench_bevy::schedule::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("kiwi", |b| {
        let mut bench = bench_kiwi::schedule::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("legion", |b| {
        let mut bench = bench_legion::schedule::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("planck_ecs", |b| {
        let mut bench = bench_planck_ecs::schedule::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("shipyard", |b| {
        let mut bench = bench_shipyard::schedule::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("specs", |b| {
        let mut bench = bench_specs::schedule::Benchmark::new();
        b.iter(move || bench.run());
    });

    group.finish();
}

fn bench_heavy_compute(c: &mut Criterion) {
    let mut group = c.benchmark_group("heavy_compute");

    group.bench_function("apecs", |b| {
        let mut bench = bench_apecs::heavy_compute::Benchmark::new().unwrap();
        b.iter(move || bench.run());
    });
    group.bench_function("bevy", |b| {
        let mut bench = bench_bevy::heavy_compute::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("hecs", |b| {
        let mut bench = bench_hecs::heavy_compute::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("kiwi", |b| {
        let mut bench = bench_kiwi::heavy_compute::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("legion", |b| {
        let mut bench = bench_legion::heavy_compute::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("shipyard", |b| {
        let mut bench = bench_shipyard::heavy_compute::Benchmark::new();
        b.iter(move || bench.run());
    });
    group.bench_function("specs", |b| {
        let mut bench = bench_specs::heavy_compute::Benchmark::new();
        b.iter(move || bench.run());
    });
}

criterion_group!(
    benches,
    bench_add_remove,
    bench_simple_iter,
    bench_simple_insert,
    bench_frag_iter,
    bench_schedule,
    bench_heavy_compute
);
criterion_main!(benches);
