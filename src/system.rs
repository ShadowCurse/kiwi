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

impl<F> SystemParameterFunction<()> for F
where
    F: Fn() + 'static,
{
    fn run(&mut self) {
        self()
    }
}

impl<F, P1> SystemParameterFunction<(P1,)> for F
where
    F: Fn(P1) + 'static,
    P1: SystemParameter,
{
    fn run(&mut self) {
        eprintln!("callig function with one system parameter is not implemented yet");
    }
}

impl<F, P1, P2> SystemParameterFunction<(P1, P2)> for F
where
    F: Fn(P1, P2) + 'static,
    P1: SystemParameter,
    P2: SystemParameter,
{
    fn run(&mut self) {
        eprintln!("callig function with two system parameters is not implemented yet");
    }
}

impl SystemParameter for () {}
impl<P> SystemParameter for (P,) where P: SystemParameter {}
impl<P1, P2> SystemParameter for (P1, P2)
where
    P1: SystemParameter,
    P2: SystemParameter,
{
}

#[cfg(test)]
mod test {
    use super::*;

    impl SystemParameter for u32 {}

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
