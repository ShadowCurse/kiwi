use kiwi::{
    component::Component,
    impl_component,
    query::Query,
    system::Systems,
    utils::types::TypeId,
    world::{World, WorldRefMut},
};

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

impl_component!(Position);

#[derive(Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

impl_component!(Velocity);

fn setup(mut world: WorldRefMut) {
    let ball1 = world.create();
    world
        .add_component(ball1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(ball1, Velocity { x: 1.0, y: 0.0 })
        .unwrap();

    let ball2 = world.create();
    world
        .add_component(ball2, Position { x: 4.0, y: 1.0 })
        .unwrap();
    world
        .add_component(ball2, Velocity { x: 1.0, y: -1.0 })
        .unwrap();

    let ball3 = world.create();
    world
        .add_component(ball3, Position { x: -3.0, y: 1.0 })
        .unwrap();
    world
        .add_component(ball3, Velocity { x: -1.0, y: 3.0 })
        .unwrap();
}

fn update_ball(query: Query<(&mut Position, &mut Velocity), 2>) {
    for (pos, vel) in query.iter() {
        pos.x += vel.x;
        pos.y += vel.y;

        if pos.x < -10.0 || 10.0 < pos.x {
            vel.x *= -1.0;
        } else if pos.y < -10.0 || 10.0 < pos.y {
            vel.y *= -1.0;
        }
    }
}

fn print_ball(query: Query<(&Position,), 1>) {
    for pos in query.iter() {
        println!("pos: {pos:?}");
    }
}

fn main() {
    let mut ecs = World::default();

    let mut systems = Systems::default();

    systems.add_startup_system(setup);
    systems.add_system(update_ball);
    systems.add_system(print_ball);

    for _ in 0..5 {
        systems.run(&mut ecs);
    }
}
