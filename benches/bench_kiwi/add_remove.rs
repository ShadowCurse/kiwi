use kiwi::{
    component::Component, entity::Entity, impl_component, utils::types::TypeId, world::World,
};

#[derive(Debug)]
struct A(f32);
impl_component!(A);

#[derive(Debug)]
struct B(f32);
impl_component!(B);

pub struct Benchmark {
    world: World,
    entities: Vec<Entity>,
}

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::default();
        let entities = (0..10000)
            .map(|_| {
                let entity = world.create();
                world.add_component(entity, A(0.0)).unwrap();
                entity
            })
            .collect();

        Self { world, entities }
    }

    pub fn run(&mut self) {
        for e in self.entities.iter() {
            self.world.add_component(*e, B(0.0)).unwrap();
        }
        for e in self.entities.iter() {
            self.world.remove_component::<B>(*e).unwrap();
        }
    }
}
