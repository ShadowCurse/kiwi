use std::marker::PhantomData;

use crate::{
    resources::{ResMut, Resource},
    system::{SystemParameter, SystemParameterFetch},
    world::World,
};

#[derive(Debug, Clone)]
pub struct Events<T: 'static> {
    pub events: Vec<T>,
}

impl<T: 'static> Default for Events<T> {
    fn default() -> Self {
        Self { events: vec![] }
    }
}

impl<T: 'static> Resource for Events<T> {}

pub struct EventReader<'a, T: 'static> {
    events: &'a Events<T>,
}

impl<'a, T: 'static> EventReader<'a, T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.events.events.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.events.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.events.len()
    }
}

impl<'a, T: 'static> SystemParameter for EventReader<'a, T> {
    type Fetch = EventReaderFetch<T>;
}

pub struct EventReaderFetch<T: 'static> {
    phantom: PhantomData<T>,
}

impl<T: 'static> SystemParameterFetch for EventReaderFetch<T> {
    type Item<'a> = EventReader<'a, T>;

    fn fetch(world: &mut World) -> Self::Item<'_> {
        Self::Item {
            events: world
                .get_resource::<Events<T>>()
                .expect("couldn't find event type"),
        }
    }
}

pub struct EventWriter<'a, T: 'static> {
    events: &'a mut Events<T>,
}

impl<'a, T: 'static> EventWriter<'a, T> {
    pub fn send(&mut self, event: T) {
        self.events.events.push(event);
    }

    pub fn is_empty(&self) -> bool {
        self.events.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.events.len()
    }
}

impl<'a, T: 'static> SystemParameter for EventWriter<'a, T> {
    type Fetch = EventWriterFetch<T>;
}

pub struct EventWriterFetch<T: 'static> {
    phantom: PhantomData<T>,
}

impl<T: 'static> SystemParameterFetch for EventWriterFetch<T> {
    type Item<'a> = EventWriter<'a, T>;

    fn fetch(world: &mut World) -> Self::Item<'_> {
        Self::Item {
            events: world
                .get_resource_mut::<Events<T>>()
                .expect("couldn't find event type"),
        }
    }
}

pub fn clear_events<T: 'static>(mut events: ResMut<Events<T>>) {
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
        struct E {
            i: u8,
        }

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
