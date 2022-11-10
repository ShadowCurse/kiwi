use std::marker::PhantomData;

pub trait System: 'static {
    fn run(&mut self);
}

pub trait SystemParameter {}

pub trait IntoSystem<Params> {
    type Output: System;

    fn into_system(self) -> Self::Output;
}

pub trait SystemParameterFunction<Parameter: SystemParameter>: 'static {
    fn run(&mut self);
}

#[derive(Default)]
pub struct Systems {
    systems: Vec<Box<dyn System>>,
}

impl Systems {
    pub fn add_system<S: IntoSystem<P>, P: SystemParameter>(&mut self, system: S) {
        self.systems.push(Box::new(system.into_system()));
    }

    pub fn run(&mut self) {
        for system in self.systems.iter_mut() {
            system.run();
        }
    }
}

impl<F, P> IntoSystem<P> for F
where
    F: SystemParameterFunction<P> + 'static,
    P: SystemParameter + 'static,
{
    type Output = FunctionSystem<F, P>;

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

impl<F, P> System for FunctionSystem<F, P>
where
    F: SystemParameterFunction<P> + 'static,
    P: SystemParameter + 'static,
{
    fn run(&mut self) {
        self.system.run()
    }
}

macro_rules! impl_system_param_func {
    ($($t:ident),*) => {
        impl<F, $($t),*> SystemParameterFunction<($($t, )*)> for F
        where
            F: Fn($($t),*) + 'static,
            $($t: SystemParameter),*,
        {
            fn run(&mut self) {

            }
        }
    };
}

impl<F> SystemParameterFunction<()> for F
where
    F: Fn() + 'static,
{
    fn run(&mut self) {
        self()
    }
}

impl_system_param_func!(P1);
impl_system_param_func!(P1, P2);
impl_system_param_func!(P1, P2, P3);
impl_system_param_func!(P1, P2, P3, P4);
impl_system_param_func!(P1, P2, P3, P4, P5);

macro_rules! impl_system_param_for_type {
    ($t:ty) => {
        impl SystemParameter for $t {}
    };
}

macro_rules! impl_system_param_tuple {
    ($($t:ident),*) => {
        impl<$($t),*> SystemParameter for ($($t, )*) where $($t: SystemParameter),*, {}
    };
}

impl_system_param_for_type!(());

impl_system_param_tuple!(P1);
impl_system_param_tuple!(P1, P2);
impl_system_param_tuple!(P1, P2, P3);
impl_system_param_tuple!(P1, P2, P3, P4);
impl_system_param_tuple!(P1, P2, P3, P4, P5);

#[cfg(test)]
mod test {
    use super::*;

    impl_system_param_for_type!(u32);

    #[test]
    fn add_systems() {
        fn test_sys() {}
        fn test_sys_u32(_: u32) {}
        fn test_sys_void_and_u32(_: (), _: u32) {}

        let mut systems = Systems::default();

        systems.add_system(test_sys);
        systems.add_system(test_sys_u32);
        systems.add_system(test_sys_void_and_u32);

        systems.run();
    }
}
