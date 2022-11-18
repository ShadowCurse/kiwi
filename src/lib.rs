#![feature(const_type_id)]
#![feature(concat_idents)]
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

use archetype::{ArchetypeId, ArchetypeInfo, Archetypes};
use component::Component;
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
    #[error("Trying to access non existing entity")]
    NonExistingEntity,
    #[error("Trying to access non existing archetype")]
    NonExistingArchetype,
    #[error("Trying to access non existing table")]
    NonExistingTable,
}

#[derive(Debug, Default)]
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
    pub fn add_component<C: Component>(
        &mut self,
        entity: Entity,
        component: C,
    ) -> Result<(), EcsError> {
        match self.entity_to_archetype.get(&entity) {
            Some(arch) => {
                let mut arch_info = match self.archetypes.get(*arch) {
                    Some(arch) => arch.clone(),
                    None => Err(EcsError::NonExistingArchetype)?,
                };
                let old_table_id = match self.archetype_to_table.get(arch) {
                    Some(table_id) => table_id,
                    None => Err(EcsError::NonExistingTable)?,
                };
                let old_arch = arch_info.archetype();
                arch_info.add_component::<C>()?;

                let new_arch_id = self.archetypes.get_or_insert(arch_info)?;

                let new_table_id = match self.archetype_to_table.get(&new_arch_id) {
                    Some(new_table_id) => *new_table_id,
                    None => self.storage.new_table(),
                };

                self.storage
                    .transfer_components(*old_table_id, new_table_id, entity, old_arch)?;
                // TODO don't forget entity
                self.storage.insert_component(new_table_id, component)?;
            }
            None => Err(EcsError::NonExistingEntity)?,
        }
        Ok(())
    }

    /// Removes component from the entity
    /// Returns error if component does not exist
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Result<(), EcsError> {
        todo!()
    }
}
