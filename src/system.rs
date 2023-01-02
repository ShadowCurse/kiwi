use std::marker::PhantomData;

use crate::world::World;

pub trait System: 'static {
    fn run(&mut self, world: &World);
}

pub trait SystemParameter: Sized {
    type Fetch: SystemParameterFetch;
}

pub trait SystemParameterFetch {
    type Item<'a>: SystemParameter<Fetch = Self>;

    fn fetch(world: &World) -> Self::Item<'_>;
}

pub type SystemParameterItem<'w, P> = <<P as SystemParameter>::Fetch as SystemParameterFetch>::Item<'w>;

pub trait IntoSystem<Params> {
    type Output: System;

    fn into_system(self) -> Self::Output;
}


pub trait SystemParameterFunction<Parameter: SystemParameter>: 'static {
    fn run(&mut self, params: SystemParameterItem<Parameter>);
}

#[derive(Default)]
pub struct Systems {
    systems: Vec<Box<dyn System>>,
}

impl Systems {
    pub fn add_system<S, P>(&mut self, system: S)
    where
        S: IntoSystem<P>,
        P: SystemParameter,
    {
        self.systems.push(Box::new(system.into_system()));
    }

    pub fn run(&mut self, world: &World) {
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
            params: PhantomData,
        }
    }
}

pub struct FunctionSystem<S, Params: SystemParameter> {
    system: S,
    params: PhantomData<Params>,
}

impl<S, P> System for FunctionSystem<S, P>
where
    S: SystemParameterFunction<P> + 'static,
    P: SystemParameter + 'static,
{
    fn run(&mut self, ecs: &World) {
        let params = P::Fetch::fetch(ecs);
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

macro_rules! impl_system_param_tuple {
    ($($t:ident),*) => {
        impl<$($t),*> SystemParameterFetch for ($($t),*,)
        where $($t: SystemParameterFetch),*,
        {
            type Item<'a> = (
                $($t::Item<'a>),*,
            );

            fn fetch<'ecs>(ecs: &'ecs World) -> Self::Item<'ecs> {
                (
                    $($t::fetch(ecs)),*
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
    type Fetch = TupleFetch;
}

pub struct TupleFetch;
impl SystemParameterFetch for TupleFetch {
    type Item<'a> = ();

    fn fetch(_ecs: &'_ World) -> Self::Item<'_> {}
}

impl_system_param_tuple!(P1);
impl_system_param_tuple!(P1, P2);
impl_system_param_tuple!(P1, P2, P3);
impl_system_param_tuple!(P1, P2, P3, P4);
impl_system_param_tuple!(P1, P2, P3, P4, P5);

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! impl_dummy_sys_param {
        ($fetch:ident, $t:tt) => {
            pub struct $fetch;
            impl SystemParameterFetch for $fetch {
                type Item<'a> = $t;
                fn fetch(_: &'_ World) -> Self::Item<'_> {
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
        fn test_sys() {
            println!("test_sys()");
        }
        fn test_sys_u32(_: u32) {
            println!("test_sys_u32(_: u32)");
        }
        fn test_sys_void_and_u32(_: (), _: u32) {
            println!("test_sys_void_and_u32(_: (), _: u32)");
        }
        fn test_sys_tuples(_: ((), u32), _: (bool, bool)) {
            println!("test_sys_tuples(_: ((), u32), _: (bool, bool))");
        }

        let ecs = World::default();

        let mut systems = Systems::default();

        systems.add_system(test_sys);
        systems.add_system(test_sys_u32);
        systems.add_system(test_sys_void_and_u32);
        systems.add_system(test_sys_tuples);

        systems.run(&ecs);
    }
}
