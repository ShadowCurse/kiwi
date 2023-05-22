use std::{any::TypeId, alloc::Layout};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct TypeInfo {
    pub id: TypeId,
    pub layout: Layout,
    pub name: &'static str,
    pub drop: Option<fn(*mut ())>,
}

impl TypeInfo {
    pub const fn new<T: TypeDrop + 'static>() -> Self {
        let drop = if std::mem::needs_drop::<Self>() {
            Some(unsafe { std::mem::transmute(&<Self as TypeDrop>::type_drop) })
        } else {
            None
        };
        Self {
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            name: std::any::type_name::<T>(),
            drop,
        }
    }
}

pub trait TypeDrop: Sized {
    /// # Safety
    /// The pointer should point to the instance of the correct type.
    unsafe fn type_drop(component: *mut ()) {
        component.cast::<Self>().drop_in_place();
    }
}

impl<T> TypeDrop for T {}

