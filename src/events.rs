use std::{fmt::Debug, marker::PhantomData};

use crate::{
    resources::{ResMut, Resource},
    system::{SystemParameter, SystemParameterFetch},
    world::World,
};

pub trait Event: Debug + 'static {}

#[derive(Debug, Clone)]
pub struct Events<E: Event> {
    pub events: Vec<E>,
}

impl<E: Event> Default for Events<E> {
    fn default() -> Self {
        Self { events: vec![] }
    }
}

impl<E: Event> Resource for Events<E> {}

#[derive(Debug)]
pub struct EventReader<'a, E: Event> {
    events: &'a Events<E>,
}

impl<'a, E: Event> EventReader<'a, E> {
    pub fn iter(&self) -> impl Iterator<Item = &E> {
        self.events.events.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.events.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.events.len()
    }
}

impl<'a, E: Event> SystemParameter for EventReader<'a, E> {
    type Fetch = EventReaderFetch<E>;
}

#[derive(Debug)]
pub struct EventReaderFetch<E: Event> {
    phantom: PhantomData<E>,
}

impl<E: Event> SystemParameterFetch for EventReaderFetch<E> {
    type Item<'a> = EventReader<'a, E>;

    fn fetch(world: &mut World) -> Self::Item<'_> {
        Self::Item {
            events: world
                .get_resource::<Events<E>>()
                .expect("couldn't find event type"),
        }
    }
}

#[derive(Debug)]
pub struct EventWriter<'a, E: Event> {
    events: &'a mut Events<E>,
}

impl<'a, E: Event> EventWriter<'a, E> {
    pub fn send(&mut self, event: E) {
        self.events.events.push(event);
    }

    pub fn is_empty(&self) -> bool {
        self.events.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.events.len()
    }
}

impl<'a, E: Event> SystemParameter for EventWriter<'a, E> {
    type Fetch = EventWriterFetch<E>;
}

#[derive(Debug)]
pub struct EventWriterFetch<E: Event> {
    phantom: PhantomData<E>,
}

impl<E: Event> SystemParameterFetch for EventWriterFetch<E> {
    type Item<'a> = EventWriter<'a, E>;

    fn fetch(world: &mut World) -> Self::Item<'_> {
        Self::Item {
            events: world
                .get_resource_mut::<Events<E>>()
                .expect("couldn't find event type"),
        }
    }
}

pub fn clear_events<E: Event>(mut events: ResMut<Events<E>>) {
    events
        .get_mut()
        .expect("couldn't find event type")
        .events
        .clear();
}

#[cfg(test)]
mod test {
    use crate::system::Systems;

    use super::*;

    #[test]
    fn events() {
        #[derive(Debug)]
        struct E {
            i: u8,
        }

        impl Event for E {}

        fn test_write_events(mut writer: EventWriter<E>) {
            for i in 0..10 {
                writer.send(E { i });
            }
        }

        fn test_read_events(reader: EventReader<E>) {
            let sum = reader.iter().map(|e| e.i).sum::<u8>();
            assert_eq!(sum, (0..10).sum());
        }

        let mut ecs = World::default();
        let mut systems = Systems::default();

        ecs.add_event::<E>();

        systems.add_system(test_write_events);
        systems.add_system(test_read_events);

        systems.run(&mut ecs);
    }
}
