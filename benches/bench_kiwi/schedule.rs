use kiwi::{
    component::Component, impl_component, query::Query, system::Systems, utils::types::TypeId,
    world::World,
};

#[derive(Debug, Copy, Clone)]
struct A(f32);
impl_component!(A);

#[derive(Debug, Copy, Clone)]
struct B(f32);
impl_component!(B);

#[derive(Debug, Copy, Clone)]
struct C(f32);
impl_component!(C);

#[derive(Debug, Copy, Clone)]
struct D(f32);
impl_component!(D);

#[derive(Debug, Copy, Clone)]
struct E(f32);
impl_component!(E);

fn ab(query: Query<(&mut A, &mut B), 2>) {
    for (a, b) in query.iter() {
        std::mem::swap(&mut a.0, &mut b.0);
    }
}

fn cd(query: Query<(&mut C, &mut D), 2>) {
    for (c, d) in query.iter() {
        std::mem::swap(&mut c.0, &mut d.0);
    }
}

fn ce(query: Query<(&mut C, &mut E), 2>) {
    for (c, e) in query.iter() {
        std::mem::swap(&mut c.0, &mut e.0);
    }
}

pub struct Benchmark {
    world: World,
    systems: Systems,
}

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::default();
        for _ in 0..10000 {
            let entity = world.create();
            world.add_component(entity, A(0.0)).unwrap();
            world.add_component(entity, B(0.0)).unwrap();
        }
        for _ in 0..10000 {
            let entity = world.create();
            world.add_component(entity, A(0.0)).unwrap();
            world.add_component(entity, B(0.0)).unwrap();
            world.add_component(entity, C(0.0)).unwrap();
        }
        for _ in 0..10000 {
            let entity = world.create();
            world.add_component(entity, A(0.0)).unwrap();
            world.add_component(entity, B(0.0)).unwrap();
            world.add_component(entity, C(0.0)).unwrap();
            world.add_component(entity, D(0.0)).unwrap();
        }
        for _ in 0..10000 {
            let entity = world.create();
            world.add_component(entity, A(0.0)).unwrap();
            world.add_component(entity, B(0.0)).unwrap();
            world.add_component(entity, C(0.0)).unwrap();
            world.add_component(entity, E(0.0)).unwrap();
        }

        let mut systems = Systems::default();
        systems.add_system(ab);
        systems.add_system(cd);
        systems.add_system(ce);

        Self { world, systems }
    }

    pub fn run(&mut self) {
        self.systems.run(&mut self.world);
    }
}
