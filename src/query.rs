use std::marker::PhantomData;

use crate::{
    component::{ComponentTuple, ComponentTupleMut},
    system::{SystemParameter, SystemParameterFetch},
    world::World,
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
    fn iter<'a>(&'a self) -> impl Iterator<Item = T> + 'a {
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

    fn fetch(world: &mut World) -> Self::Item<'_> {
        Self::Item {
            world,
            phantom: PhantomData,
        }
    }
}

pub struct QueryMut<'world, T, const L: usize>
where
    T: ComponentTupleMut<L>,
{
    world: &'world mut World,
    phantom: PhantomData<T>,
}

impl<'ecs, T, const L: usize> QueryMut<'ecs, T, L>
where
    T: ComponentTupleMut<L> + 'ecs,
{
    fn iter<'a>(&'a self) -> impl Iterator<Item = T> + 'a {
        self.world.query_mut::<T, L>()
    }
}

impl<'a, T, const L: usize> SystemParameter for QueryMut<'a, T, L>
where
    T: ComponentTupleMut<L>,
{
    type Fetch = QueryFetchMut<T, L>;
}

pub struct QueryFetchMut<T, const L: usize>
where
    T: ComponentTupleMut<L>,
{
    phantom: PhantomData<T>,
}

impl<T, const L: usize> SystemParameterFetch for QueryFetchMut<T, L>
where
    T: ComponentTupleMut<L>,
{
    type Item<'a> = QueryMut<'a, T, L>;

    fn fetch(world: &mut World) -> Self::Item<'_> {
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

        let mut ecs = World::default();

        let mut systems = Systems::default();

        systems.add_system(test_sys_query);

        systems.run(&mut ecs);
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

        systems.run(&mut ecs);
    }

    #[test]
    fn query_mut_in_ecs() {
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

        fn query_u8_u16_u32_mutate(query: QueryMut<(&mut u8, &mut u16, &mut u32), 3>) {
            for (_u8, _u16, _u32) in query.iter() {
                *_u8 += 1;
                *_u16 += 1;
                *_u32 += 1;
            }
        }

        fn query_u8_u16_u32_check(query: Query<(&u8, &u16, &u32), 3>) {
            let mut results = query.iter().collect::<Vec<_>>();
            results.sort_unstable();
            let expected = [(&2, &3, &4), (&5, &6, &7)];
            assert_eq!(results, expected);
        }

        let mut systems = Systems::default();

        systems.add_system(query_u8_u16_u32_mutate);
        systems.add_system(query_u8_u16_u32_check);

        systems.run(&mut ecs);
    }
}
