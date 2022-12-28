use std::marker::PhantomData;

use crate::Ecs;

pub trait System: 'static {
    fn run(&mut self, ecs: &Ecs);
}

pub trait SystemParameter: Sized {
    type Fetch<'a>: SystemParameterFetch;
}

pub trait SystemParameterFetch {
    type Item<'a>: SystemParameter<Fetch<'a> = Self>;

    fn fetch<'ecs>(ecs: &'ecs Ecs) -> Self::Item<'_>;
}

pub trait IntoSystem<Params> {
    type Output: System;

    fn into_system(self) -> Self::Output;
}

pub trait SystemParameterFunction<Parameter: SystemParameter>: 'static {
    fn run(&mut self, params: Parameter);
}

#[derive(Default)]
pub struct Systems {
    systems: Vec<Box<dyn System>>,
}

impl Systems {
    pub fn add_system<S, P, F>(&mut self, system: S)
    where
        S: IntoSystem<P>,
        P: for<'b> SystemParameter<Fetch<'b> = F>,
        F: for<'a> SystemParameterFetch<Item<'a> = P>,
    {
        self.systems.push(Box::new(system.into_system()));
    }

    pub fn run(&mut self, ecs: &Ecs) {
        for system in self.systems.iter_mut() {
            system.run(ecs);
        }
    }
}

impl<S, P, F> IntoSystem<P> for S
where
    S: SystemParameterFunction<P> + 'static,
    P: for<'b> SystemParameter<Fetch<'b> = F> + 'static,
    F: for<'a> SystemParameterFetch<Item<'a> = P>,
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

impl<S, P, F> System for FunctionSystem<S, P>
where
    S: SystemParameterFunction<P> + 'static,
    P: for<'b> SystemParameter<Fetch<'b> = F> + 'static,
    F: for<'a> SystemParameterFetch<Item<'a> = P>,
{
    fn run(&mut self, ecs: &Ecs) {
        let params = P::Fetch::fetch(ecs);
        self.system.run(params);
    }
}

macro_rules! impl_system_param_func {
    ($($t:ident),*) => {
        impl<F, $($t),*> SystemParameterFunction<($($t, )*)> for F
        where
            F: Fn($($t),*) + 'static,
            $($t: SystemParameter),*,
        {
            fn run(&mut self, params: ($($t, )*)) {
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

            fn fetch<'ecs>(ecs: &'ecs Ecs) -> Self::Item<'ecs> {
                (
                    $($t::fetch(ecs)),*
                    ,
                )
            }
        }

        impl<$($t),*> SystemParameter for ($($t),*,)
        where $($t: SystemParameter),*,
        {
            type Fetch<'a> = ($($t::Fetch<'a>,)*);
        }
    };
}

impl SystemParameter for () {
    type Fetch<'a> = TupleFetch;
}

pub struct TupleFetch;
impl SystemParameterFetch for TupleFetch {
    type Item<'a> = ();

    fn fetch(_ecs: &'_ Ecs) -> Self::Item<'_> {}
}

impl_system_param_tuple!(P1);
impl_system_param_tuple!(P1, P2);
impl_system_param_tuple!(P1, P2, P3);
impl_system_param_tuple!(P1, P2, P3, P4);
impl_system_param_tuple!(P1, P2, P3, P4, P5);

#[cfg(test)]
mod test {
    use super::*;

    pub struct BoolFetch;
    impl SystemParameterFetch for BoolFetch {
        type Item<'a> = bool;
        fn fetch(_: &'_ Ecs) -> Self::Item<'_> {
            true
        }
    }

    impl SystemParameter for bool {
        type Fetch<'a> = BoolFetch;
    }

    pub struct U32Fetch;
    impl SystemParameterFetch for U32Fetch {
        type Item<'a> = u32;
        fn fetch(_: &'_ Ecs) -> Self::Item<'_> {
            0
        }
    }
    impl SystemParameter for u32 {
        type Fetch<'a> = U32Fetch;
    }

    #[test]
    fn add_and_run_systems() {
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

        let ecs = Ecs::default();

        let mut systems = Systems::default();

        systems.add_system(test_sys);
        systems.add_system(test_sys_u32);
        systems.add_system(test_sys_void_and_u32);
        systems.add_system(test_sys_tuples);

        systems.run(&ecs);
    }
}
