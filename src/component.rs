use std::fmt::Debug;

use crate::{
    blobvec::BlobVec,
    count_tts,
    entity::Entity,
    utils::{static_sort, types::TypeId},
};

trait ComponentRef {
    type Component: Component;

    fn fetch(blob: &BlobVec, line: usize) -> Self;
}

impl<T> ComponentRef for &T
where
    T: Component,
{
    type Component = T;

    fn fetch(blob: &BlobVec, line: usize) -> Self {
        unsafe { &*blob.get_ptr::<T>(line) }
    }
}

impl<T> ComponentRef for &mut T
where
    T: Component,
{
    type Component = T;

    fn fetch(blob: &BlobVec, line: usize) -> Self {
        unsafe { &mut *blob.get_ptr_mut(line) }
    }
}

pub trait Component: Sized + Debug + 'static {
    const ID: TypeId;
}

#[macro_export]
macro_rules! impl_component {
    ($t:tt) => {
        impl Component for $t {
            const ID: TypeId = TypeId::of::<$t>();
        }
    };
}

impl_component!(bool);
impl_component!(u8);
impl_component!(u16);
impl_component!(u32);
impl_component!(u64);
impl_component!(u128);
impl_component!(i8);
impl_component!(i16);
impl_component!(i32);
impl_component!(i64);
impl_component!(i128);
impl_component!(f32);
impl_component!(f64);

pub trait ComponentTuple<const L: usize>: Sized + Debug + 'static {
    const IDS: [TypeId; L];
    const SORTED_IDS: [TypeId; L];

    fn fetch(entity: Entity, columns: &[&BlobVec; L], line: usize) -> Self;
}

macro_rules! impl_component_tuple {
    ($($t:ident),*) => {
        impl<$($t),*> ComponentTuple<{count_tts!($($t)*)}> for ($($t,)*)
        where
            $($t: Debug + 'static, $t: ComponentRef, <$t as ComponentRef>::Component: Component),*,
        {
            const IDS: [TypeId; count_tts!($($t)*)] = [
                $(
                    TypeId::of::<<$t as ComponentRef>::Component>()
                ),*
            ];

            const SORTED_IDS: [TypeId; count_tts!($($t)*)] = {
                let ids: [TypeId; count_tts!($($t)*)] = [
                    $(
                        TypeId::of::<<$t as ComponentRef>::Component>()
                    ),*
                ];
                static_sort(ids, 0, count_tts!($($t)*) as isize - 1)
            };

            fn fetch(_entity: Entity, columns: &[&BlobVec; {count_tts!($($t)*)}], line: usize) -> Self {
                let mut _index = 0;
                (
                    $(
                        {
                            let a = $t::fetch(columns[_index], line);
                            _index += 1;
                            a
                        }
                    ),*,
                )
            }
        }
    };
}

impl_component_tuple!(C1);
impl_component_tuple!(C1, C2);
impl_component_tuple!(C1, C2, C3);
impl_component_tuple!(C1, C2, C3, C4);
impl_component_tuple!(C1, C2, C3, C4, C5);
impl_component_tuple!(C1, C2, C3, C4, C5, C6);
impl_component_tuple!(C1, C2, C3, C4, C5, C6, C7);
impl_component_tuple!(C1, C2, C3, C4, C5, C6, C7, C8);
impl_component_tuple!(C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_component_tuple!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);

macro_rules! impl_component_tuple_with_entity {
    ($($t:ident),*) => {
        impl<$($t),*> ComponentTuple<{count_tts!($($t)*)}> for (Entity, $($t,)*)
        where
            $($t: Debug + 'static, $t: ComponentRef, <$t as ComponentRef>::Component: Component),*,
        {
            const IDS: [TypeId; count_tts!($($t)*)] = [
                $(
                    TypeId::of::<<$t as ComponentRef>::Component>()
                ),*
            ];

            const SORTED_IDS: [TypeId; count_tts!($($t)*)] = {
                let ids: [TypeId; count_tts!($($t)*)] = [
                    $(
                        TypeId::of::<<$t as ComponentRef>::Component>()
                    ),*
                ];
                static_sort(ids, 0, count_tts!($($t)*) as isize - 1)
            };

            fn fetch(entity: Entity, columns: &[&BlobVec; {count_tts!($($t)*)}], line: usize) -> Self {
                let mut _index = 0;
                (
                    entity,
                    $(
                        {
                            let a = $t::fetch(columns[_index], line);
                            _index += 1;
                            a
                        }
                    ),*,
                )
            }
        }
    };
}

impl_component_tuple_with_entity!(C1);
impl_component_tuple_with_entity!(C1, C2);
impl_component_tuple_with_entity!(C1, C2, C3);
impl_component_tuple_with_entity!(C1, C2, C3, C4);
impl_component_tuple_with_entity!(C1, C2, C3, C4, C5);
impl_component_tuple_with_entity!(C1, C2, C3, C4, C5, C6);
impl_component_tuple_with_entity!(C1, C2, C3, C4, C5, C6, C7);
impl_component_tuple_with_entity!(C1, C2, C3, C4, C5, C6, C7, C8);
impl_component_tuple_with_entity!(C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_component_tuple_with_entity!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn components_component_tuple_ids() {
        let mut expected = [
            TypeId::of::<u8>(),
            TypeId::of::<bool>(),
            TypeId::of::<i32>(),
        ];
        expected.sort_unstable();
        assert_eq!(<(&u8, &bool, &i32)>::SORTED_IDS, expected);
        assert_eq!(<(&bool, &u8, &i32)>::SORTED_IDS, expected);
        assert_eq!(<(&i32, &bool, &u8)>::SORTED_IDS, expected);

        assert_eq!(<(&mut u8, &bool, &i32)>::SORTED_IDS, expected);
        assert_eq!(<(&bool, &mut u8, &i32)>::SORTED_IDS, expected);
        assert_eq!(<(&i32, &bool, &mut u8)>::SORTED_IDS, expected);

        assert_eq!(<(&mut i32, &mut bool, &mut u8)>::SORTED_IDS, expected);
    }
}
