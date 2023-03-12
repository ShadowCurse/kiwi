use std::{any::TypeId, collections::HashMap};

use crate::{blobvec::BlobVec, utils::TypeInfo};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ResourceError {
    #[error("Attemt to remove non existing resource (type id: {0})")]
    RemoveNonExisting(&'static str),
    #[error("Attemt to get non existing resource (type id: {0})")]
    GetNonExisting(&'static str),
}

pub trait Resource: 'static {}

#[derive(Debug, Default)]
pub struct Resources {
    columns: HashMap<TypeId, BlobVec>,
}

impl Resources {
    pub fn add<T: Resource>(&mut self, resource: T) {
        let type_info = TypeInfo::new::<T>();
        match self.columns.get_mut(&type_info.id) {
            Some(column) => {
                // If column exist for the type
                // then it is safe to swap previous instance
                // with new one.
                // Old instance will be dropped here.
                unsafe {
                    let _ = column.swap(0, resource);
                }
            }
            None => {
                let mut blob = BlobVec::new(type_info.layout, type_info.drop);

                // Safe becaus blob has the correct type
                unsafe { blob.push(resource) };

                self.columns.insert(type_info.id, blob);
            }
        }
    }

    pub fn remove<T: Resource>(&mut self) -> Result<(), ResourceError> {
        let type_info = TypeInfo::new::<T>();
        self.columns
            .remove(&type_info.id)
            .map(|mut column| {
                // If column exist for the type
                // then it is safe to swap previous instance
                // with new one.
                // Old instance will be dropped here.
                unsafe {
                    column.drop_at(0);
                }
            })
            .ok_or(ResourceError::RemoveNonExisting(type_info.name))
    }

    pub fn get<T: Resource>(&self) -> Result<&T, ResourceError> {
        let type_info = TypeInfo::new::<T>();
        match self.columns.get(&type_info.id) {
            Some(column) => {
                // Safe because column contains a corret type
                Ok(unsafe { column.get::<T>(0) })
            }
            None => Err(ResourceError::GetNonExisting(type_info.name)),
        }
    }

    pub fn get_mut<T: Resource>(&mut self) -> Result<&T, ResourceError> {
        let type_info = TypeInfo::new::<T>();
        match self.columns.get_mut(&type_info.id) {
            Some(column) => {
                // Safe because column contains a corret type
                Ok(unsafe { column.get_mut::<T>(0) })
            }
            None => Err(ResourceError::GetNonExisting(type_info.name)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct A {
        val: u8,
    }
    impl Resource for A {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct B {
        val: u16,
    }
    impl Resource for B {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct C {
        val: u32,
    }
    impl Resource for C {}

    #[test]
    fn resources_add_get() {
        let mut resources = Resources::default();

        let a = A { val: 1 };
        resources.add(a);

        let b = B { val: 2 };
        resources.add(b);

        let get_a = resources.get::<A>().unwrap();
        assert_eq!(&a, get_a);

        let get_b = resources.get::<B>().unwrap();
        assert_eq!(&b, get_b);

        let get_c = resources.get::<C>().unwrap_err();
        assert_eq!(
            get_c,
            ResourceError::GetNonExisting("kiwi::resources::test::C")
        );
    }

    #[test]
    fn resources_add_remove_get() {
        let mut resources = Resources::default();

        let a = A { val: 1 };
        resources.add(a);

        let b = B { val: 2 };
        resources.add(b);

        assert!(resources.remove::<A>().is_ok());

        let get_a = resources.get::<A>().unwrap_err();
        assert_eq!(
            get_a,
            ResourceError::GetNonExisting("kiwi::resources::test::A")
        );

        let get_b = resources.get::<B>().unwrap();
        assert_eq!(&b, get_b);

        let get_c = resources.get::<C>().unwrap_err();
        assert_eq!(
            get_c,
            ResourceError::GetNonExisting("kiwi::resources::test::C")
        );

        assert_eq!(
            resources.remove::<C>().unwrap_err(),
            ResourceError::RemoveNonExisting("kiwi::resources::test::C")
        );
    }
}
