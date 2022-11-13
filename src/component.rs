use std::{alloc::Layout, any::TypeId};

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
