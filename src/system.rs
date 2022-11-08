use std::marker::PhantomData;

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

pub trait System {
    fn run(&mut self);
}

pub trait SystemParameter {}

pub trait IntoSystem<Params> {
    type Output: System + 'static;

    fn into_system(self) -> Self::Output;
}

pub struct FunctionSystem<S, Params: SystemParameter> {
    system: S,
    params: PhantomData<Params>,
}

pub trait SystemParameterFunction<Parameter: SystemParameter>: 'static {
    fn run(&mut self);
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

impl<F, P> System for FunctionSystem<F, P>
where
    F: SystemParameterFunction<P> + 'static,
    P: SystemParameter,
{
    fn run(&mut self) {
        self.system.run()
    }
}
