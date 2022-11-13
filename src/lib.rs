#![feature(const_type_id)]
#![feature(fn_traits)]

mod archetype;
mod blobvec;
mod component;
mod entity;
mod query;
mod sparse_set;
mod system;
mod table;
mod utils;

use std::collections::HashMap;

use archetype::{Archetype, ArchetypeId, Archetypes};
use entity::{Entity, EntityGenerator};
use table::{TableId, TableStorage};

#[derive(Debug, thiserror::Error)]
pub enum EcsError {
    #[error("Adding component dublicate to the archetype")]
    AddingComponentDuplicate,
    #[error("Removing non existing component form the archetype")]
    RemovingNonExistingComponent,
    #[error("Inserting archetype dublicate in component trie")]
    InsertingArchetypeDuplicate,
    #[error("Removing non existing archetype from component trie")]
    RemovingNonExistingArchetype,
    #[error("Table does not exist in the TableStorage")]
    TableDoesNotExist,
    #[error("Table does not contain component column")]
    TableDoesNotContainComponentColumn,
    #[error("Table already has column for this type")]
    TableRegisteringDuplicatedComponent,
    #[error("Table already has the archetype assigned to it")]
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
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Result<(), EcsError> {
        todo!()
    }

    // pub fn query(&self, query: Archetype) -> impl Iterator<Item = ()> {
    //     todo!()
    // }
}
