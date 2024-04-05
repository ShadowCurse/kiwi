use cgmath::*;
use kiwi::{component::Component, impl_component, utils::types::TypeId, world::World};
use rayon::prelude::*;

#[derive(Debug, Copy, Clone)]
struct Transform(Matrix4<f32>);
impl_component!(Transform);

#[derive(Debug, Copy, Clone)]
struct Position(Vector3<f32>);
impl_component!(Position);

#[derive(Debug, Copy, Clone)]
struct Rotation(Vector3<f32>);
impl_component!(Rotation);

#[derive(Debug, Copy, Clone)]
struct Velocity(Vector3<f32>);
impl_component!(Velocity);

pub struct Benchmark {
    world: World,
}

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::default();
        for _ in 0..1000 {
            let entity = world.create();
            world
                .add_component(entity, Transform(Matrix4::<f32>::from_angle_x(Rad(1.2))))
                .unwrap();
            world
                .add_component(entity, Position(Vector3::unit_x()))
                .unwrap();
            world
                .add_component(entity, Rotation(Vector3::unit_x()))
                .unwrap();
            world
                .add_component(entity, Velocity(Vector3::unit_x()))
                .unwrap();
        }

        Self { world }
    }

    pub fn run(&mut self) {
        let query = self.world.query::<(&mut Position, &mut Transform), 2>();
        query.par_bridge().for_each(|(pos, mat)| {
            use cgmath::Transform;
            for _ in 0..100 {
                mat.0 = mat.0.invert().unwrap();
            }

            pos.0 = mat.0.transform_vector(pos.0);
        });
    }
}
