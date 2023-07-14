use kiwi::{
    component::Component,
    query::Query,
    system::Systems,
    world::{World, WorldRefMut},
};

#[derive(Debug)]
struct Position<const N: usize> {
    x: f32,
    y: f32,
}

impl<const N: usize> Component for Position<N> {}

#[derive(Debug)]
struct Velocity<const N: usize> {
    x: f32,
    y: f32,
}

impl<const N: usize> Component for Velocity<N> {}

macro_rules! setup_fn {
    ($name:ident, $($t:expr),*) => {
        #[tracing::instrument(skip_all)]
        fn $name(mut world: WorldRefMut) {
            for _ in 0..100 {
                let entity = world.create();
                $(world
                    .add_component(entity, Position::<$t> { x: 1.0, y: 1.0 })
                    .unwrap();
                world
                    .add_component(entity, Velocity::<$t> { x: 1.0, y: 0.0 })
                    .unwrap();
                )*
            }
        }
    };
}

setup_fn!(setup_0, 0);
setup_fn!(setup_0_1, 0, 1);
setup_fn!(setup_0_1_2, 0, 1, 2);
setup_fn!(setup_0_1_2_3, 0, 1, 2, 3);

macro_rules! update_fn {
    ($name:ident, $total:expr, $($t:expr),*) => {
        #[tracing::instrument(skip_all)]
        fn $name(
            query: Query<
                (
                    $(
                      &mut Position<$t>,
                      &mut Velocity<$t>,
                    )*
                ),
                $total,
            >,
        ) {
            for item in query.iter() {
                $(
                item.0.x += item.1.x + $t as f32;
                item.1.y += item.0.y;
                )*
            }
        }
    };
}

update_fn!(update_0, 2, 0);
update_fn!(update_0_1, 4, 0, 1);
update_fn!(update_0_1_2, 6, 0, 1, 2);
update_fn!(update_0_1_2_3, 8, 0, 1, 2, 3);

fn setup_global_subscriber() -> (impl Drop, impl Drop) {
    use tracing_chrome::ChromeLayerBuilder;
    use tracing_flame::FlameLayer;
    use tracing_subscriber::{fmt, prelude::*};

    let fmt_layer = fmt::Layer::default();

    let (flame_layer, guard_falme) = FlameLayer::with_file("./tracing.folded").unwrap();
    let (chrome_layer, guard_chrome) = ChromeLayerBuilder::new().build();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(flame_layer)
        .with(chrome_layer)
        .init();
    (guard_falme, guard_chrome)
}

fn main() {
    let _guards = setup_global_subscriber();

    let mut ecs = World::default();

    let mut systems = Systems::default();

    systems.add_startup_system(setup_0);
    systems.add_startup_system(setup_0_1);
    systems.add_startup_system(setup_0_1_2);
    systems.add_startup_system(setup_0_1_2_3);
    systems.add_system(update_0);
    systems.add_system(update_0_1);
    systems.add_system(update_0_1_2);
    systems.add_system(update_0_1_2_3);

    for _ in 0..500 {
        systems.run(&mut ecs);
    }
}
