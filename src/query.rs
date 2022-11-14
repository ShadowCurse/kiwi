use std::{any::TypeId, marker::PhantomData};

use crate::{component::Component, count_tts, system::SystemParameter, utils::static_sort, Ecs};

pub trait TupleIds<const L: usize> {
    const IDS: [TypeId; L];
    fn ids() -> &'static [TypeId] {
        &Self::IDS
    }
}

struct Query<'esc, T> {
    esc: &'esc Ecs,
    phantom: PhantomData<T>,
}

impl<T> Query<'_, T> {
    pub fn iter(&self) -> impl Iterator<Item = T> {
        todo!()
    }
}

impl<T> SystemParameter for Query<'_, T> {
    fn new(esc: &Ecs) -> Self {
        Query {
            esc,
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
}
