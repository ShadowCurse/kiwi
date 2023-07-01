use std::fmt::{Display, Formatter};

pub const MAX_ENTITIES: u16 = std::u16::MAX;

type EntityGeneration = u16;
type EntityId = u16;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Entity {
    id: u16,
    gen: u16,
}

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl Entity {
    pub fn from_raw(id: u16, gen: u16) -> Self {
        Entity { id, gen }
    }
}

#[derive(Debug, Default)]
pub struct EntityGenerator {
    generations: Vec<EntityGeneration>,
    pending: Vec<EntityId>,
}

impl EntityGenerator {
    pub fn new() -> Self {
        Self {
            generations: Vec::with_capacity(MAX_ENTITIES as usize),
            pending: Vec::with_capacity(MAX_ENTITIES as usize),
        }
    }

    pub fn create(&mut self) -> Entity {
        if let Some(id) = self.pending.pop() {
            Entity {
                id,
                gen: self.generations[id as usize],
            }
        } else {
            let id = self.generations.len() as u16;
            self.generations.push(0);
            Entity { id, gen: 0 }
        }
    }

    pub fn delete(&mut self, e: &Entity) {
        let generation = self.generations.get_mut(e.id as usize).unwrap();
        if *generation != e.gen {
            return;
        }
        *generation += 1;
        self.pending.push(e.id);
    }
}
