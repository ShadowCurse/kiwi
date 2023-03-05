use kiwi::{component::Component, query::Query, system::Systems, world::World};

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

    let ball1 = ecs.create();
    ecs.add_component(ball1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    ecs.add_component(ball1, Velocity { x: 1.0, y: 0.0 })
        .unwrap();

    let ball2 = ecs.create();
    ecs.add_component(ball2, Position { x: 4.0, y: 1.0 })
        .unwrap();
    ecs.add_component(ball2, Velocity { x: 1.0, y: -1.0 })
        .unwrap();

    let ball3 = ecs.create();
    ecs.add_component(ball3, Position { x: -3.0, y: 1.0 })
        .unwrap();
    ecs.add_component(ball3, Velocity { x: -1.0, y: 3.0 })
        .unwrap();

    let mut systems = Systems::default();

    systems.add_system(update_ball);
    systems.add_system(print_ball);

    for _ in 0..5 {
        systems.run(&mut ecs);
    }
}
