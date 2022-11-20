use std::any::TypeId;

use crate::system::{IntoSystem, System, SystemParameter};

struct Schedule {
    systems: Vec<Box<dyn System>>,
}

impl Schedule {
    /// Adds system that is run on every [`Systems::run`] call;
    #[tracing::instrument(skip_all)]
    pub fn add_system<S, P>(&mut self, system: S)
    where
        S: IntoSystem<P>,
        P: SystemParameter,
    {
        self.systems.push(Box::new(system.into_system()));
    }
}
