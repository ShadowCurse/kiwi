use std::{alloc::Layout, any::TypeId};

use crate::{
    count_tts,
    tuple_from_array,
    utils::static_sort,
};

pub trait Component: 'static {}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ComponentInfo {
    pub id: TypeId,
    pub layout: Layout,
}

impl ComponentInfo {
    pub fn new<T: Component>() -> Self {
        Self {
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
        }
    }
}

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

pub trait FlattenTuple {
    type Flatten;

    fn flatten(self) -> Self::Flatten;
}

impl<C1> FlattenTuple for (C1,) {
    type Flatten = (C1,);

    fn flatten(self) -> Self::Flatten {
        self
    }
}

impl<C1, C2> FlattenTuple for (C1, (C2,)) {
    type Flatten = (C1, C2);

    fn flatten(self) -> Self::Flatten {
        (self.0, self.1 .0)
    }
}

impl<C1, C2, C3> FlattenTuple for (C1, (C2, (C3,))) {
    type Flatten = (C1, C2, C3);

    fn flatten(self) -> Self::Flatten {
        (self.0, self.1 .0, self.1 .1 .0)
    }
}

impl<C1, C2, C3, C4> FlattenTuple for (C1, (C2, (C3, (C4,)))) {
    type Flatten = (C1, C2, C3, C4);

    fn flatten(self) -> Self::Flatten {
        (self.0, self.1 .0, self.1 .1 .0, self.1 .1 .1 .0)
    }
}

impl<C1, C2, C3, C4, C5> FlattenTuple for (C1, (C2, (C3, (C4, (C5,))))) {
    type Flatten = (C1, C2, C3, C4, C5);

    fn flatten(self) -> Self::Flatten {
        (
            self.0,
            self.1 .0,
            self.1 .1 .0,
            self.1 .1 .1 .0,
            self.1 .1 .1 .1 .0,
        )
    }
}

pub trait ComponentTuple<const L: usize> {
    const IDS: [TypeId; L];

    fn ids() -> [TypeId; L] {
        Self::IDS
    }

    fn ids_ref() -> &'static [TypeId] {
        &Self::IDS
    }

    unsafe fn from_erased_ref_array(array: [&(); L]) -> Self;
}

pub trait ComponentTupleMut<const L: usize> {
    const IDS: [TypeId; L];

    fn ids() -> [TypeId; L] {
        Self::IDS
    }

    fn ids_ref() -> &'static [TypeId] {
        &Self::IDS
    }

    unsafe fn from_erased_ref_array_mut(array: [*mut (); L]) -> Self;
}

macro_rules! impl_component_tuple {
    ($($t:ident),*) => {
        impl<$($t),*> ComponentTuple<{count_tts!($($t)*)}> for ($(&$t,)*)
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

            unsafe fn from_erased_ref_array(array: [&(); count_tts!($($t)*)]) -> Self {
                const L:usize = count_tts!($($t)*);
                tuple_from_array!(L, array, $($t,)*).flatten()
            }
        }

        impl<$($t),*> ComponentTupleMut<{count_tts!($($t)*)}> for ($(&mut $t,)*)
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

            unsafe fn from_erased_ref_array_mut(array: [*mut (); count_tts!($($t)*)]) -> Self {
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
    }
}
