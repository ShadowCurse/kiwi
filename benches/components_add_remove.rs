use kiwi::{component::Component, entity::Entity, world::World};

use criterion::{criterion_group, criterion_main, Criterion};

#[derive(Debug)]
struct Position {
    _x: f32,
    _y: f32,
}

impl Component for Position {}

fn update(world: &mut World, entities: &[Entity]) {
    for e in entities {
        world
            .add_component(*e, Position { _x: 1.0, _y: 1.0 })
            .unwrap();
    }
    for e in entities {
        world.remove_component::<Position>(*e).unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut ecs = World::default();
    let entities = (0..10000).map(|_| ecs.create()).collect::<Vec<_>>();

    c.bench_function("components_add_remove", |b| b.iter(|| update(&mut ecs, &entities)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
