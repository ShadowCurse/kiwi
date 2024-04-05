use kiwi::{component::Component, impl_component, utils::types::TypeId, world::World};

#[derive(Debug, Copy, Clone)]
struct Data(f32);
impl_component!(Data);

macro_rules! create_entities {
    ($world:ident; $( $variants:ident ),*) => {
        $(
            #[derive(Debug)]
            struct $variants(f32);
            impl Component for $variants {
                const ID: TypeId = TypeId::of::<$variants>();
            }
            for _ in 0..20 {
                let entity = $world.create();
                $world
                    .add_component(entity, $variants(0.0))
                    .unwrap();
                $world
                    .add_component(entity, Data(1.0))
                    .unwrap();
            }
        )*
    };
}

pub struct Benchmark {
    world: World,
}

impl Benchmark {
    pub fn new() -> Self {
        let mut world = World::default();

        create_entities!(world; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

        Self { world }
    }

    pub fn run(&mut self) {
        for (data,) in self.world.query::<(&mut Data,), 1>() {
            data.0 *= 2.0;
        }
    }
}
