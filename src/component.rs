use std::any::TypeId;

use crate::{
    count_tts,
    entity::Entity,
    tuple_from_array,
    utils::{flat_tuple::FlattenTuple, static_sort},
};

trait ComponentRef {
    type Component: Component;

    fn from_raw_ptr(raw: *mut ()) -> Self;
}

impl<T> ComponentRef for &T
where
    T: Component,
{
    type Component = T;

    fn from_raw_ptr(r: *mut ()) -> Self {
        unsafe { &*(r as *const T) }
    }
}

impl<T> ComponentRef for &mut T
where
    T: Component,
{
    type Component = T;

    fn from_raw_ptr(r: *mut ()) -> Self {
        unsafe { &mut *(r as *mut T) }
    }
}

pub trait Component: Sized + 'static {}

macro_rules! impl_component {
    ($t:tt) => {
        impl Component for $t {}
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

pub trait ComponentTuple<const L: usize>: Sized {
    const IDS: [TypeId; L];

    fn ids() -> [TypeId; L] {
        Self::IDS
    }

    fn ids_ref() -> &'static [TypeId] {
        &Self::IDS
    }

    /// # Safety
    /// Pointers in the array should be of the correct types
    unsafe fn from_erased_mut_ptr_array(_array: [*mut (); L]) -> Self;
}

pub trait ComponentTupleWithEntity<const L: usize>: Sized {
    const IDS: [TypeId; L];

    fn ids() -> [TypeId; L] {
        Self::IDS
    }

    fn ids_ref() -> &'static [TypeId] {
        &Self::IDS
    }

    /// # Safety
    /// Pointers in the array should be of the correct types
    unsafe fn from_erased_mut_ptr_array_with_entity(_: (Entity, [*mut (); L])) -> Self;
}

macro_rules! impl_component_tuple {
    ($($t:ident),*) => {
        impl<$($t),*> ComponentTuple<{count_tts!($($t)*)}> for ($($t,)*)
        where
            $($t: 'static, $t: ComponentRef, <$t as ComponentRef>::Component: Component),*,
        {
            // TODO
            // Currently we transform TypeId to u64, but this is wrond
            // Wait untill TypId can be compared in compile time and change this mess
            const IDS: [TypeId; count_tts!($($t)*)] = {
                let ids_u64: [u64; count_tts!($($t)*)] = [
                    $(
                        unsafe {
                            std::mem::transmute::<_, u64>(TypeId::of::<<$t as ComponentRef>::Component>())
                        }
                    ),*
                ];
                let ids_u64 = static_sort(ids_u64, 0, count_tts!($($t)*) as isize - 1);
                let mut ids_type_id: [TypeId; count_tts!($($t)*)] = [$(TypeId::of::<<$t as ComponentRef>::Component>()),*];
                let mut _index = 0;
                $(
                    let _ = TypeId::of::<$t>();
                    ids_type_id[_index] = unsafe { std::mem::transmute::<_, TypeId>(ids_u64[_index]) };
                    _index += 1;
                )*
               ids_type_id
            };

            unsafe fn from_erased_mut_ptr_array(array: [*mut (); count_tts!($($t)*)]) -> Self {
                const L:usize = count_tts!($($t)*);
                tuple_from_array!(L, array, $($t,)*).flatten()
            }
        }
    };
}

impl_component_tuple!(C1);
impl_component_tuple!(C1, C2);
impl_component_tuple!(C1, C2, C3);
impl_component_tuple!(C1, C2, C3, C4);
impl_component_tuple!(C1, C2, C3, C4, C5);

macro_rules! impl_component_tuple_with_entity {
    ($($t:ident),*) => {
        impl<$($t),*> ComponentTupleWithEntity<{count_tts!($($t)*)}> for (Entity, ($($t,)*))
        where
            $($t: 'static, $t: ComponentRef, <$t as ComponentRef>::Component: Component),*,
        {
            // TODO
            // Currently we transform TypeId to u64, but this is wrond
            // Wait untill TypId can be compared in compile time and change this mess
            const IDS: [TypeId; count_tts!($($t)*)] = {
                let ids_u64: [u64; count_tts!($($t)*)] = [
                    $(
                        unsafe {
                            std::mem::transmute::<_, u64>(TypeId::of::<<$t as ComponentRef>::Component>())
                        }
                    ),*
                ];
                let ids_u64 = static_sort(ids_u64, 0, count_tts!($($t)*) as isize - 1);
                let mut ids_type_id: [TypeId; count_tts!($($t)*)] = [$(TypeId::of::<<$t as ComponentRef>::Component>()),*];
                let mut _index = 0;
                $(
                    let _ = TypeId::of::<$t>();
                    ids_type_id[_index] = unsafe { std::mem::transmute::<_, TypeId>(ids_u64[_index]) };
                    _index += 1;
                )*
               ids_type_id
            };

            unsafe fn from_erased_mut_ptr_array_with_entity((entity, array): (Entity, [*mut (); count_tts!($($t)*)])) -> Self {
                const L:usize = count_tts!($($t)*);
                (entity, tuple_from_array!(L, array, $($t,)*).flatten())
            }
        }
    };
}

impl_component_tuple_with_entity!(C1);
impl_component_tuple_with_entity!(C1, C2);
impl_component_tuple_with_entity!(C1, C2, C3);
impl_component_tuple_with_entity!(C1, C2, C3, C4);
impl_component_tuple_with_entity!(C1, C2, C3, C4, C5);

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use super::*;

    #[test]
    fn components_component_tuple_ids() {
        let mut expected = [
            TypeId::of::<u8>(),
            TypeId::of::<bool>(),
            TypeId::of::<i32>(),
        ];
        expected.sort_unstable();
        assert_eq!(<(&u8, &bool, &i32)>::ids(), expected);
        assert_eq!(<(&bool, &u8, &i32)>::ids(), expected);
        assert_eq!(<(&i32, &bool, &u8)>::ids(), expected);

        assert_eq!(<(&mut u8, &bool, &i32)>::ids(), expected);
        assert_eq!(<(&bool, &mut u8, &i32)>::ids(), expected);
        assert_eq!(<(&i32, &bool, &mut u8)>::ids(), expected);

        assert_eq!(<(&mut i32, &mut bool, &mut u8)>::ids(), expected);
    }

    #[test]
    fn components_convert() {
        let a = 1;
        let b = 1.1;
        let c = false;

        let array: [*mut (); 3] = [
            &a as *const i32 as *mut (),
            &b as *const f64 as *mut (),
            &c as *const bool as *mut (),
        ];
        let q = unsafe {
            <(&mut i32, &mut f64, &mut bool) as ComponentTuple<3>>::from_erased_mut_ptr_array(array)
        };
        *q.0 += 1;
        *q.2 = true;
        assert_eq!(*q.0, 2);
        assert_eq!(*q.1, 1.1);
        assert!(*q.2);
    }
}
