use std::marker::PhantomData;

use bumpalo::Bump;

use crate::{
    component::ComponentTuple,
    system::{SystemParameter, SystemParameterCache, SystemParameterFetch},
    world::World,
};

pub struct Query<'world, 'cache, T, const L: usize>
where
    T: ComponentTuple<L> + 'static,
{
    world: &'world World,
    cache: &'cache <QueryFetch<T, L> as SystemParameterFetch>::Cache,
    phantom: PhantomData<T>,
}

impl<'world, 'cache, T, const L: usize> Query<'world, 'cache, T, L>
where
    T: ComponentTuple<L>,
{
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.world.query_with_cache::<T, L>(self.cache)
    }
}

impl<'world, 'cache, T, const L: usize> SystemParameter for Query<'world, 'cache, T, L>
where
    T: ComponentTuple<L>,
{
    type Fetch = QueryFetch<T, L>;
}

#[derive(Debug)]
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
    type Item<'world, 'cache> = Query<'world, 'cache, T, L>;
    type Cache = QueryCache;

    fn fetch<'world, 'cache>(
        world: &'world mut World,
        cache: &'cache Self::Cache,
    ) -> Self::Item<'world, 'cache> {
        Self::Item {
            world,
            cache,
            phantom: PhantomData,
        }
    }
}

pub struct QueryCache {
    pub allocator: Bump,
}

impl SystemParameterCache for QueryCache {
    fn empty() -> Self {
        Self {
            allocator: Bump::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{entity::Entity, system::Systems};

    use super::*;

    #[test]
    fn query_system_param() {
        fn test_sys_query(q: Query<(&u8, &bool), 2>) {
            let _ = q.iter();
        }

        let mut ecs = World::default();

        let mut systems = Systems::default();

        systems.add_system(test_sys_query);

        systems.run(&mut ecs);
    }

    #[test]
    fn query_with_entity_system_param() {
        fn test_sys_query(q: Query<(Entity, &u8, &bool), 2>) {
            let _ = q.iter();
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

        fn query_u8_u16_u32_mutate(query: Query<(&mut u8, &mut u16, &mut u32), 3>) {
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
