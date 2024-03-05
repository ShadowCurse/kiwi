use std::{any::TypeId, collections::HashMap, fmt::Debug, marker::PhantomData};

use crate::{
    blobvec::BlobVec,
    system::{SystemParameter, SystemParameterFetch},
    utils::type_traits::TypeInfo,
    world::World,
};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Attemt to remove non existing resource (type id: {0})")]
    RemoveNonExisting(&'static str),
    #[error("Attemt to get non existing resource (type id: {0})")]
    GetNonExisting(&'static str),
}

pub trait Resource: Debug + 'static {}

#[derive(Debug)]
pub struct Res<'world, T>
where
    T: Resource,
{
    world: &'world World,
    phantom: PhantomData<T>,
}

impl<T> Res<'_, T>
where
    T: Resource,
{
    pub fn get(&self) -> Result<&T, crate::world::Error> {
        self.world.get_resource()
    }
}

impl<'a, T> SystemParameter for Res<'a, T>
where
    T: Resource,
{
    type Fetch = ResFetch<T>;
}

#[derive(Debug)]
pub struct ResFetch<T>
where
    T: Resource,
{
    phantom: PhantomData<T>,
}

impl<T> SystemParameterFetch for ResFetch<T>
where
    T: Resource,
{
    type Item<'world, 'cache> = Res<'world, T>;
    type Cache = ();

    fn fetch<'world, 'cache>(
        world: &'world mut World,
        _: &'cache Self::Cache,
    ) -> Self::Item<'world, 'cache> {
        Self::Item {
            world,
            phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct ResMut<'world, T>
where
    T: Resource,
{
    world: &'world mut World,
    phantom: PhantomData<T>,
}

impl<T> ResMut<'_, T>
where
    T: Resource,
{
    pub fn get_mut(&mut self) -> Result<&mut T, crate::world::Error> {
        self.world.get_resource_mut()
    }
}

impl<'a, T> SystemParameter for ResMut<'a, T>
where
    T: Resource,
{
    type Fetch = ResMutFetch<T>;
}

#[derive(Debug)]
pub struct ResMutFetch<T>
where
    T: Resource,
{
    phantom: PhantomData<T>,
}

impl<T> SystemParameterFetch for ResMutFetch<T>
where
    T: Resource,
{
    type Item<'world, 'cache> = ResMut<'world, T>;
    type Cache = ();

    fn fetch<'world, 'cache>(
        world: &'world mut World,
        _: &'cache Self::Cache,
    ) -> Self::Item<'world, 'cache> {
        Self::Item {
            world,
            phantom: PhantomData,
        }
    }
}

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

    pub fn remove<T: Resource>(&mut self) -> Result<(), Error> {
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
            .ok_or(Error::RemoveNonExisting(type_info.name))
    }

    pub fn get<T: Resource>(&self) -> Result<&T, Error> {
        let type_info = TypeInfo::new::<T>();
        match self.columns.get(&type_info.id) {
            Some(column) => {
                // Safe because column contains a corret type
                Ok(unsafe { column.get::<T>(0) })
            }
            None => Err(Error::GetNonExisting(type_info.name)),
        }
    }

    pub fn get_mut<T: Resource>(&mut self) -> Result<&mut T, Error> {
        let type_info = TypeInfo::new::<T>();
        match self.columns.get_mut(&type_info.id) {
            Some(column) => {
                // Safe because column contains a corret type
                Ok(unsafe { column.get_mut::<T>(0) })
            }
            None => Err(Error::GetNonExisting(type_info.name)),
        }
    }

    /// # Safety
    /// Save as long as same resource is accessed only once
    pub unsafe fn get_mut_unchecked<T: Resource>(&self) -> Result<&mut T, Error> {
        let type_info = TypeInfo::new::<T>();
        match self.columns.get(&type_info.id) {
            Some(column) => {
                // Safe because column contains a corret type
                Ok(unsafe { column.get_mut_unchecked::<T>(0) })
            }
            None => Err(Error::GetNonExisting(type_info.name)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::system::Systems;

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
        assert_eq!(get_c, Error::GetNonExisting("kiwi::resources::test::C"));
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
        assert_eq!(get_a, Error::GetNonExisting("kiwi::resources::test::A"));

        let get_b = resources.get::<B>().unwrap();
        assert_eq!(&b, get_b);

        let get_c = resources.get::<C>().unwrap_err();
        assert_eq!(get_c, Error::GetNonExisting("kiwi::resources::test::C"));

        assert_eq!(
            resources.remove::<C>().unwrap_err(),
            Error::RemoveNonExisting("kiwi::resources::test::C")
        );
    }

    #[test]
    fn res_system_param() {
        fn test_sys_res(_: Res<A>) {
            println!("test_sys_res(_: Res<A>)");
        }

        let mut ecs = World::default();

        let mut systems = Systems::default();

        systems.add_system(test_sys_res);

        systems.run(&mut ecs);
    }

    #[test]
    fn res_mut_system_param() {
        fn test_sys_res_mut(_: ResMut<A>) {
            println!("test_sys_res_mut(_: ResMut<A>)");
        }

        let mut ecs = World::default();

        let mut systems = Systems::default();

        systems.add_system(test_sys_res_mut);

        systems.run(&mut ecs);
    }

    #[test]
    fn res_in_ecs() {
        let mut ecs = World::default();

        let a = A { val: 1 };
        ecs.add_resource(a);

        let b = B { val: 2 };
        ecs.add_resource(b);

        fn get_a(res: Res<A>) {
            let a = res.get().unwrap();
            assert_eq!(a.val, 1);
        }

        fn get_b(res: Res<B>) {
            let b = res.get().unwrap();
            assert_eq!(b.val, 2);
        }

        fn get_c(res: Res<C>) {
            assert_eq!(
                res.get().unwrap_err(),
                crate::world::Error::Resources(Error::GetNonExisting("kiwi::resources::test::C"))
            );
        }

        let mut systems = Systems::default();

        systems.add_system(get_a);
        systems.add_system(get_b);
        systems.add_system(get_c);

        systems.run(&mut ecs);
    }

    #[test]
    fn res_mut_in_ecs() {
        let mut ecs = World::default();

        let a = A { val: 1 };
        ecs.add_resource(a);

        let b = B { val: 2 };
        ecs.add_resource(b);

        fn mutate_a(mut res: ResMut<A>) {
            let a = res.get_mut().unwrap();
            a.val = 11;
        }

        fn mutate_b(mut res: ResMut<B>) {
            let b = res.get_mut().unwrap();
            b.val = 22;
        }

        fn mutate_c(mut res: ResMut<C>) {
            assert_eq!(
                res.get_mut().unwrap_err(),
                crate::world::Error::Resources(Error::GetNonExisting("kiwi::resources::test::C"))
            );
        }

        fn validate_a(res: Res<A>) {
            let a = res.get().unwrap();
            assert_eq!(a.val, 11);
        }

        fn validate_b(res: Res<B>) {
            let b = res.get().unwrap();
            assert_eq!(b.val, 22);
        }

        let mut systems = Systems::default();

        systems.add_system(mutate_a);
        systems.add_system(mutate_b);
        systems.add_system(mutate_c);
        systems.add_system(validate_a);
        systems.add_system(validate_b);

        systems.run(&mut ecs);
    }
}
