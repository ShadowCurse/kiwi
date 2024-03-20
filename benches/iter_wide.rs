use kiwi::{
    component::Component,
    query::Query,
    system::{IntoSystem, System},
    utils::types::TypeId,
    world::{World, WorldRefMut},
};

use criterion::{criterion_group, criterion_main, Criterion};

#[derive(Debug)]
struct Position<const N: usize> {
    x: f32,
    y: f32,
}

impl<const N: usize> Component for Position<N> {
    const ID: TypeId = TypeId::of::<Position<N>>();
}

#[derive(Debug)]
struct Velocity<const N: usize> {
    x: f32,
    y: f32,
}

impl<const N: usize> Component for Velocity<N> {
    const ID: TypeId = TypeId::of::<Velocity<N>>();
}

fn setup(mut world: WorldRefMut) {
    for _ in 0..10000 {
        let entity = world.create();
        world
            .add_component(entity, Position::<0> { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity, Velocity::<0> { x: 1.0, y: 0.0 })
            .unwrap();
        world
            .add_component(entity, Position::<1> { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity, Velocity::<1> { x: 1.0, y: 0.0 })
            .unwrap();
        world
            .add_component(entity, Position::<2> { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity, Velocity::<2> { x: 1.0, y: 0.0 })
            .unwrap();
        world
            .add_component(entity, Position::<3> { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity, Velocity::<3> { x: 1.0, y: 0.0 })
            .unwrap();
        world
            .add_component(entity, Position::<4> { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity, Velocity::<4> { x: 1.0, y: 0.0 })
            .unwrap();

        world
            .add_component(entity, Position::<5> { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity, Velocity::<5> { x: 1.0, y: 0.0 })
            .unwrap();
    }
}

fn update(
    query: Query<
        (
            &mut Position<0>,
            &mut Velocity<0>,
            &mut Position<1>,
            &mut Velocity<1>,
            &mut Position<2>,
            &mut Velocity<2>,
            &mut Position<3>,
            &mut Velocity<3>,
            &mut Position<4>,
            &mut Velocity<4>,
        ),
        10,
    >,
) {
    for item in query.iter() {
        item.0.x += item.1.x;
        item.0.y += item.1.y;
        item.2.x += item.3.x;
        item.2.y += item.3.y;
        item.4.x += item.5.x;
        item.4.y += item.5.y;
        item.6.x += item.7.x;
        item.6.y += item.7.y;
        item.8.x += item.9.x;
        item.8.y += item.9.y;
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut ecs = World::default();
    setup.into_system().run(&mut ecs);
    let mut update = update.into_system();

    c.bench_function("iter_wide", |b| b.iter(|| update.run(&mut ecs)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
