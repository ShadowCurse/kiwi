use std::marker::PhantomData;

use crate::{
    archetype::Archetype,
    component::ComponentTuple,
    system::{SystemParameter, SystemParameterFetch},
    Ecs,
};

pub struct Query<'ecs, T, const L: usize>
where
    T: ComponentTuple<L>,
{
    ecs: &'ecs Ecs,
    phantom: PhantomData<T>,
}

// impl<'ecs, T, const L: usize> Query<'ecs, T, L>
// where
//     T: ComponentTuple<L>,
// {
//     fn iter(&self) -> impl Iterator<Item = T> {
//         let archetype: Archetype = T::ids_ref().into();
//         let inner_iter = self.ecs.query::<L, T>(&archetype);
//         QueryIter { inner_iter, phantom: PhantomData }
//     }
// }

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

    fn fetch(ecs: &Ecs) -> Self::Item<'_> {
        Self::Item {
            ecs,
            phantom: PhantomData,
        }
    }
}

struct QueryIter<'ecs, I, T, const L: usize>
where
    I: Iterator<Item = [&'ecs (); L]>,
    T: ComponentTuple<L>,
{
    inner_iter: I,
    phantom: PhantomData<T>,
}

impl<'ecs, I, T, const L: usize> QueryIter<'ecs, I, T, L>
where
    I: Iterator<Item = [&'ecs (); L]>,
    T: ComponentTuple<L>,
{
    pub fn new(inner_iter: I) -> Self {
        Self {
            inner_iter,
            phantom: PhantomData,
        }
    }
}

impl<'ecs, I, T, const L: usize> Iterator for QueryIter<'ecs, I, T, L>
where
    I: Iterator<Item = [&'ecs (); L]>,
    T: ComponentTuple<L>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter
            .next()
            .map(|array| unsafe { T::from_erased_ref_array(array) })
    }
}

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use crate::system::Systems;

    use super::*;

    #[test]
    fn query() {
        let mut expected = [
            TypeId::of::<u8>(),
            TypeId::of::<bool>(),
            TypeId::of::<i32>(),
        ];
        expected.sort_unstable();
        assert_eq!(<(&u8, &bool, &i32)>::ids(), expected);
        assert_eq!(<(&bool, &u8, &i32)>::ids(), expected);
        assert_eq!(<(&i32, &bool, &u8)>::ids(), expected);
    }

    #[test]
    fn query_in_system() {
        fn test_sys_query(_: Query<(&u8, &bool), 2>) {
            println!("test_sys(_: Query::<(&u8, &bool)>)");
        }

        let ecs = Ecs::default();

        let mut systems = Systems::default();

        systems.add_system(test_sys_query);

        systems.run(&ecs);
    }
}
