use kiwi::{
    component::Component,
    query::Query,
    system::{IntoSystem, System},
    world::{World, WorldRefMut},
};

use criterion::{criterion_group, criterion_main, Criterion};

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

impl Component for Position {}

struct Velocity {
    x: f32,
    y: f32,
}

impl Component for Velocity {}

fn setup(mut world: WorldRefMut) {
    for _ in 0..10000 {
        let entity = world.create();
        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity, Velocity { x: 1.0, y: 0.0 })
            .unwrap();
    }
}

fn update(query: Query<(&mut Position, &mut Velocity), 2>) {
    for (pos, vel) in query.iter() {
        pos.x += vel.x;
        pos.y += vel.y;
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut ecs = World::default();
    setup.into_system().run(&mut ecs);
    let mut update = update.into_system();

    c.bench_function("iter_simple", |b| b.iter(|| update.run(&mut ecs)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
