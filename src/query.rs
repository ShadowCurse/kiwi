use std::marker::PhantomData;

use crate::{
    component::ComponentTuple,
    system::{SystemParameter, SystemParameterFetch}, world::World,
};

pub struct Query<'world, T, const L: usize>
where
    T: ComponentTuple<L>,
{
    world: &'world World,
    phantom: PhantomData<T>,
}

impl<'ecs, T, const L: usize> Query<'ecs, T, L>
where
    T: ComponentTuple<L> + 'ecs,
{
    fn iter(&self) -> impl Iterator<Item = T> + 'ecs {
        self.world.query::<T, L>()
    }
}

impl<'a, T, const L: usize> SystemParameter for Query<'a, T, L>
where
    T: ComponentTuple<L>,
{
    type Fetch = QueryFetch<T, L>;
}

pub struct QueryFetch<T, const L: usize>
where
    T: ComponentTuple<L>,
{
    phantom: PhantomData<T>,
}

impl<T, const L: usize> SystemParameterFetch for QueryFetch<T, L>
where
    T: ComponentTuple<L>,
{
    type Item<'a> = Query<'a, T, L>;

    fn fetch(world: &World) -> Self::Item<'_> {
        Self::Item {
            world,
            phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::system::Systems;

    use super::*;

    #[test]
    fn query_system_param() {
        fn test_sys_query(_: Query<(&u8, &bool), 2>) {
            println!("test_sys(_: Query::<(&u8, &bool)>)");
        }

        let ecs = World::default();

        let mut systems = Systems::default();

        systems.add_system(test_sys_query);

        systems.run(&ecs);
    }

    #[test]
    fn query_in_ecs() {
        let mut ecs = World::default();

        let entity = ecs.create();
        ecs.add_component(entity, 1u8).unwrap();
        ecs.add_component(entity, 2u16).unwrap();
        ecs.add_component(entity, 3u32).unwrap();

        let entity2 = ecs.create();
        ecs.add_component(entity2, 4u8).unwrap();
        ecs.add_component(entity2, 5u16).unwrap();
        ecs.add_component(entity2, 6u32).unwrap();

        let entity3 = ecs.create();
        ecs.add_component(entity3, 7u8).unwrap();
        ecs.add_component(entity3, 8u16).unwrap();
        ecs.add_component(entity3, 9u64).unwrap();

        fn query_u8(query: Query<(&u8,), 1>) {
            let mut results = query.iter().collect::<Vec<_>>();
            results.sort_unstable();
            let expected = [(&1,), (&4,), (&7,)];
            assert_eq!(results, expected);
        }

        fn query_u8_u16(query: Query<(&u8, &u16), 2>) {
            let mut results = query.iter().collect::<Vec<_>>();
            results.sort_unstable();
            let expected = [(&1, &2), (&4, &5), (&7, &8)];
            assert_eq!(results, expected);
        }

        fn query_u8_u16_u32(query: Query<(&u8, &u16, &u32), 3>) {
            let mut results = query.iter().collect::<Vec<_>>();
            results.sort_unstable();
            let expected = [(&1, &2, &3), (&4, &5, &6)];
            assert_eq!(results, expected);
        }

        let mut systems = Systems::default();

        systems.add_system(query_u8);
        systems.add_system(query_u8_u16);
        systems.add_system(query_u8_u16_u32);

        systems.run(&ecs);
    }
}
