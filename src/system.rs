use std::{fmt::Debug, marker::PhantomData};

use crate::world::World;

pub trait System: 'static {
    fn run(&mut self, world: &mut World);
}

pub trait SystemParameter: Sized {
    type Fetch: SystemParameterFetch;
}

pub trait SystemParameterFetch {
    type Item<'world, 'cache>: SystemParameter<Fetch = Self>;
    type Cache: SystemParameterCache;

    fn fetch<'world, 'cache>(
        world: &'world mut World,
        cache: &'cache Self::Cache,
    ) -> Self::Item<'world, 'cache>;
}

pub trait SystemParameterCache {
    fn empty() -> Self;
}

pub type SystemParameterItem<'world, 'cache, P> =
    <<P as SystemParameter>::Fetch as SystemParameterFetch>::Item<'world, 'cache>;

pub trait IntoSystem<Params> {
    type Output: System;

    fn into_system(self) -> Self::Output;
}

pub trait SystemParameterFunction<Parameter: SystemParameter>: 'static {
    fn run(&mut self, params: SystemParameterItem<Parameter>);
}

pub struct Systems {
    is_startup: bool,
    startup_systems: Vec<Box<dyn System>>,
    systems: Vec<Box<dyn System>>,
}

impl Default for Systems {
    fn default() -> Self {
        Self {
            is_startup: true,
            startup_systems: Default::default(),
            systems: Default::default(),
        }
    }
}

impl Debug for Systems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "is_startup: {}, startup_systems num: {}, system num: {}",
            self.is_startup,
            self.startup_systems.len(),
            self.systems.len()
        ))
    }
}

impl Systems {
    /// Adds system that is run only on startup
    #[tracing::instrument(skip_all)]
    pub fn add_startup_system<S, P>(&mut self, system: S)
    where
        S: IntoSystem<P>,
        P: SystemParameter,
    {
        self.startup_systems.push(Box::new(system.into_system()));
    }

    /// Adds system that is run on every [`Systems::run`] call;
    #[tracing::instrument(skip_all)]
    pub fn add_system<S, P>(&mut self, system: S)
    where
        S: IntoSystem<P>,
        P: SystemParameter,
    {
        self.systems.push(Box::new(system.into_system()));
    }

    /// Runs all the systems
    #[tracing::instrument(skip_all)]
    pub fn run(&mut self, world: &mut World) {
        if self.is_startup {
            for system in self.startup_systems.iter_mut() {
                system.run(world);
            }
            self.is_startup = false;
        }

        for system in self.systems.iter_mut() {
            system.run(world);
        }
    }
}

impl<S, P> IntoSystem<P> for S
where
    S: SystemParameterFunction<P> + 'static,
    P: SystemParameter + 'static,
{
    type Output = FunctionSystem<S, P>;

    fn into_system(self) -> Self::Output {
        Self::Output {
            system: self,
            cache: <<P as SystemParameter>::Fetch as SystemParameterFetch>::Cache::empty(),
            params: PhantomData,
        }
    }
}

pub struct FunctionSystem<S, Params: SystemParameter> {
    system: S,
    cache: <<Params as SystemParameter>::Fetch as SystemParameterFetch>::Cache,
    params: PhantomData<Params>,
}

impl<S, P> System for FunctionSystem<S, P>
where
    S: SystemParameterFunction<P> + 'static,
    P: SystemParameter + 'static,
{
    #[tracing::instrument(skip_all)]
    fn run(&mut self, ecs: &mut World) {
        let params = P::Fetch::fetch(ecs, &self.cache);
        self.system.run(params);
    }
}

macro_rules! impl_system_param_func {
    ($($t:ident),*) => {
        impl<F, $($t),*> SystemParameterFunction<($($t, )*)> for F
        where
            F: Fn($($t),*) + 'static,
            F: Fn($(SystemParameterItem<$t>),*) + 'static,
            $($t: SystemParameter),*,
        {
            fn run(&mut self, params: SystemParameterItem<($($t, )*)>) {
                // TODO
                // maybe try with tuple unpacking
                // to avoid relying of nightly features
                self.call(params);
            }
        }
    };
}

impl<F> SystemParameterFunction<()> for F
where
    F: Fn() + 'static,
{
    fn run(&mut self, _: ()) {
        self()
    }
}

impl_system_param_func!(P1);
impl_system_param_func!(P1, P2);
impl_system_param_func!(P1, P2, P3);
impl_system_param_func!(P1, P2, P3, P4);
impl_system_param_func!(P1, P2, P3, P4, P5);
impl_system_param_func!(P1, P2, P3, P4, P5, P6);
impl_system_param_func!(P1, P2, P3, P4, P5, P6, P7);

macro_rules! impl_system_param_tuple {
    ($(($t:ident, $i:tt)),*) => {
        impl<$($t),*> SystemParameterFetch for ($($t),*,)
        where $($t: SystemParameterFetch),*,
        {
            type Item<'world, 'cache> = (
                $($t::Item<'world, 'cache>),*,
            );
            type Cache = (
                $($t::Cache),*,
            );
            fn fetch<'world, 'cache>(
                ecs: &'world mut World,
                cache: &'cache <Self as SystemParameterFetch>::Cache
            ) -> Self::Item<'world, 'cache> {
                (
                    $($t::fetch(unsafe {  &mut *(ecs as *mut World) }, &cache.$i )),*
                    ,
                )
            }
        }

        impl<$($t),*> SystemParameterCache for ($($t),*,)
        where $($t: SystemParameterCache),*,
        {
            fn empty() -> Self {
                (
                    $($t::empty()),*
                    ,
                )
            }
        }

        impl<$($t),*> SystemParameter for ($($t),*,)
        where $($t: SystemParameter),*,
        {
            type Fetch = ($($t::Fetch,)*);
        }
    };
}

impl SystemParameter for () {
    type Fetch = ();
}

impl SystemParameterFetch for () {
    type Item<'world, 'cache> = ();
    type Cache = ();

    fn fetch<'world, 'cache>(
        _ecs: &'world mut World,
        _cache: &'cache (),
    ) -> Self::Item<'world, 'cache> {
    }
}

impl SystemParameterCache for () {
    fn empty() -> Self {}
}

impl_system_param_tuple!((P1, 0));
impl_system_param_tuple!((P1, 0), (P2, 1));
impl_system_param_tuple!((P1, 0), (P2, 1), (P3, 2));
impl_system_param_tuple!((P1, 0), (P2, 1), (P3, 2), (P4, 3));
impl_system_param_tuple!((P1, 0), (P2, 1), (P3, 2), (P4, 3), (P5, 4));
impl_system_param_tuple!((P1, 0), (P2, 1), (P3, 2), (P4, 3), (P5, 4), (P6, 5));
impl_system_param_tuple!(
    (P1, 0),
    (P2, 1),
    (P3, 2),
    (P4, 3),
    (P5, 4),
    (P6, 5),
    (P7, 6)
);

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! impl_dummy_sys_param {
        ($fetch:ident, $t:tt) => {
            pub struct $fetch;
            impl SystemParameterFetch for $fetch {
                type Item<'world, 'cache> = $t;
                type Cache = ();
                fn fetch<'world, 'cache>(
                    _: &'world mut World,
                    _: &'cache Self::Cache,
                ) -> Self::Item<'world, 'cache> {
                    Default::default()
                }
            }
            impl SystemParameter for $t {
                type Fetch = $fetch;
            }
        };
    }

    impl_dummy_sys_param!(BoolFetch, bool);
    impl_dummy_sys_param!(U8Fetch, u8);
    impl_dummy_sys_param!(U16Fetch, u16);
    impl_dummy_sys_param!(U32Fetch, u32);
    impl_dummy_sys_param!(U64Fetch, u64);

    #[test]
    fn systems_add_and_run_systems() {
        static mut VAR: u64 = 0;

        fn test_sys() {
            unsafe { VAR += 1 };
        }
        fn test_sys_u32(_: u32) {
            unsafe { VAR += 1 };
        }
        fn test_sys_void_and_u32(_: (), _: u32) {
            unsafe { VAR += 1 };
        }
        fn test_sys_tuples(_: ((), u32), _: (bool, bool)) {
            unsafe { VAR += 1 };
        }

        let mut ecs = World::default();

        let mut systems = Systems::default();

        systems.add_system(test_sys);
        systems.add_system(test_sys_u32);
        systems.add_system(test_sys_void_and_u32);
        systems.add_system(test_sys_tuples);

        systems.run(&mut ecs);

        assert_eq!(unsafe { VAR }, 4);
    }

    #[test]
    fn systems_add_and_run_startup_systems() {
        static mut VAR: u64 = 0;

        fn test_sys() {
            unsafe { VAR += 1 };
        }

        fn test_startup_sys() {
            unsafe { VAR += 1 };
        }

        let mut ecs = World::default();

        let mut systems = Systems::default();

        systems.add_startup_system(test_startup_sys);
        systems.add_system(test_sys);

        systems.run(&mut ecs);
        assert_eq!(unsafe { VAR }, 2);

        systems.run(&mut ecs);
        assert_eq!(unsafe { VAR }, 3);
    }
}
