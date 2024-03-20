use kiwi::{component::Component, impl_component, utils::types::TypeId, world::World};

use criterion::{criterion_group, criterion_main, Criterion};

#[derive(Debug)]
struct Position {
    _x: f32,
    _y: f32,
}

impl_component!(Position);

fn update() {
    let mut ecs = World::default();
    let entities = (0..10000).map(|_| ecs.create()).collect::<Vec<_>>();
    for e in entities {
        ecs.add_component(e, Position { _x: 1.0, _y: 1.0 }).unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("components_insert", |b| b.iter(update));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
