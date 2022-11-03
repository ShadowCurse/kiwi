mod archetype;
mod entity;
mod sparse_set;
mod table;
mod blobvec;

use std::collections::HashMap;

use archetype::{Archetype, ArchetypeId, Archetypes};
use entity::{Entity, EntityGenerator};
use table::{TableId, TableStorage};

pub enum EcsError {
    AddingComponentDuplicate,
    RemovingNonExistingComponent,
    InsertingArchetypeDuplicate,
    RemovingNonExistingArchetype,
    TableDoesNotExist,
    TableDoesNotContainComponentColumn,
    TableRegisteringDuplicatedComponent,
    TableAlreadyAssignedArchetype,
}

pub struct Ecs {
    entity_generator: EntityGenerator,
    archetypes: Archetypes,
    storage: TableStorage,
    /// Mapping of entities to their archetypes
    entity_to_archetype: HashMap<Entity, ArchetypeId>,
    /// Mapping of archetypes to their tables
    archetype_to_table: HashMap<ArchetypeId, TableId>,
}

impl Ecs {
    /// Creates new entity without components
    pub fn create(&mut self) -> Entity {
        todo!()
    }

    /// Adds component to the entity
    /// Returns error if component with the same type is already added to the entity
    pub fn add_component<T: 'static>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), EcsError> {
        todo!()
    }

    /// Removes component from the entity
    /// Returns error if component does not exist 
    pub fn remove_component<T: 'static>(
        &mut self,
        entity: Entity,
    ) -> Result<(), EcsError> {
        todo!()
    }

    // pub fn query(&self, query: Archetype) -> impl Iterator<Item = ()> {
    //     todo!()
    // }
}
