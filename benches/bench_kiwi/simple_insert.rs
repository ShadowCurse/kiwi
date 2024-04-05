use cgmath::*;
use kiwi::{component::Component, impl_component, utils::types::TypeId, world::World};

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

pub struct Benchmark;

impl Benchmark {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) {
        let mut world = World::default();
        for _ in 0..10000 {
            let entity = world.create();
            world
                .add_component(entity, Transform(Matrix4::from_scale(1.0)))
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
    }
}
