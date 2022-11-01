use std::any::TypeId;
use std::num::NonZeroUsize;

use crate::EcsError;
use crate::sparse_set::SparseSet;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ArchetypeId(NonZeroUsize);

#[derive(Debug, Default)]
pub struct Archetype {
    components: Vec<TypeId>,
}

impl Archetype {
    pub fn add_component<T: 'static>(&mut self) -> Result<(), EcsError> {
        let component_id = std::any::TypeId::of::<T>();
        self.components.push(component_id);
        self.components.sort_unstable();
        Ok(())
    }
}

pub struct Archetypes {
    archetypes: SparseSet<Archetype>,
    component_trie: ComponentTrie,
}

impl Archetypes {
    pub fn insert(&self, archetype: Archetype) -> ArchetypeId {
        todo!()
    }

    pub fn search(&self, archetype: &Archetype) -> bool {
        todo!()
    }

    pub fn get(&self, archetype_id: ArchetypeId) -> Archetype {
        todo!()
    }

    pub fn get_id(&self, archetype: &Archetype) -> Option<ArchetypeId> {
        todo!()
    }
}

pub struct ComponentTrie {
    root_nodes: Vec<ComponentNode>,
}

pub struct ComponentNode {
    component: TypeId,
    /// index into ['Archetypes::archetypes'] 
    archetype: Option<ArchetypeId>,
    following_components: Vec<ComponentNode>,
}
