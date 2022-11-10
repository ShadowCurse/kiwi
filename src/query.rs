use std::{any::TypeId, marker::PhantomData};

use crate::system::SystemParameter;

pub trait ComponentTuple<const L: usize> {
    const IDS: [TypeId; L];
    fn ids() -> &'static [TypeId] {
        &Self::IDS
    }
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts 1usize))*};
}

macro_rules! impl_component {
    ($($t:ident),*) => {
        impl<$($t),*> ComponentTuple<{count_tts!($($t)*)}> for ($($t,)*)
        where
            $($t: 'static),*,
        {
            const IDS: [TypeId; count_tts!($($t)*)] = [$(TypeId::of::<$t>()),*];
        }
    };
}

impl_component!(C1);
impl_component!(C1, C2);
impl_component!(C1, C2, C4);
impl_component!(C1, C2, C4, C5);
impl_component!(C1, C2, C4, C5, C6);

#[derive(Debug, Default)]
pub struct Query<T: ComponentTuple<L>, const L: usize> {
    phantom: PhantomData<T>,
}

impl<T: ComponentTuple<L>, const L: usize> SystemParameter for Query<T, L> {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn query() {
        let _query = Query::<(u8, bool, i32), 3>::default();
    }
}
