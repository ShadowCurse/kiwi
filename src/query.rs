use std::{any::TypeId, marker::PhantomData};

use crate::{
    component::Component,
    count_tts,
    system::{SystemParameter, SystemParameterFetch},
    utils::static_sort,
    Ecs,
};

pub trait TupleIds<const L: usize> {
    const IDS: [TypeId; L];

    fn ids() -> [TypeId; L] {
        Self::IDS
    }

    fn ids_ref() -> &'static [TypeId] {
        &Self::IDS
    }
}

pub struct Query<'ecs, T> {
    ecs: &'ecs Ecs,
    phantom: PhantomData<T>,
}

impl<'a, T> SystemParameter for Query<'a, T>
{
    type Fetch = QueryFetch<T>;
}

pub struct QueryFetch<T>(PhantomData<T>);

impl<T> SystemParameterFetch for QueryFetch<T> {
    type Item<'a> = Query<'a, T>;

    fn fetch(ecs: &Ecs) -> Self::Item<'_> {
        Self::Item {
            ecs,
            phantom: PhantomData,
        }
    }
}

struct QueryIter<Iter, T>
where
    Iter: Iterator<Item = T>,
{
    inner_iter: Iter,
}

impl<Iter, T> Iterator for QueryIter<Iter, T>
where
    Iter: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next()
    }
}

macro_rules! impl_tuple_ids_for_query {
    ($($t:ident),*) => {
        impl<$($t),*> TupleIds<{count_tts!($($t)*)}> for Query<'_, ($($t,)*)>
        where
            $($t: Component),*,
        {
            // TODO
            // Currently we transform TypeId to u64, but this is wrond
            // Wait untill TypId can be compared in compile time and change this mess
            const IDS: [TypeId; count_tts!($($t)*)] = {
                let ids_u64: [u64; count_tts!($($t)*)] = [$(unsafe { std::mem::transmute::<_, u64>(TypeId::of::<$t>()) }),*];
                let ids_u64 = static_sort(ids_u64, 0, count_tts!($($t)*) as isize - 1);
                let mut ids_type_id: [TypeId; count_tts!($($t)*)] = [$(TypeId::of::<$t>()),*];
                let mut _index = 0;
                $(
                    let _ = TypeId::of::<$t>();
                    ids_type_id[_index] = unsafe { std::mem::transmute::<_, TypeId>(ids_u64[_index]) };
                    _index += 1;
                )*
               ids_type_id
            };
        }

    };
}

impl_tuple_ids_for_query!(C1);
impl_tuple_ids_for_query!(C1, C2);
impl_tuple_ids_for_query!(C1, C2, C4);
impl_tuple_ids_for_query!(C1, C2, C4, C5);
impl_tuple_ids_for_query!(C1, C2, C4, C5, C6);

#[cfg(test)]
mod test {
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
        assert_eq!(Query::<(u8, bool, i32)>::ids(), expected);
        assert_eq!(Query::<(bool, u8, i32)>::ids(), expected);
        assert_eq!(Query::<(i32, bool, u8)>::ids(), expected);
    }

    #[test]
    fn query_in_system() {
        fn test_sys_query(_: Query<(u8, bool)>) {
            println!("test_sys(_: Query::<(u8, bool)>)");
        }

        let ecs = Ecs::default();

        let mut systems = Systems::default();

        systems.add_system(test_sys_query);

        systems.run(&ecs);
    }
}
