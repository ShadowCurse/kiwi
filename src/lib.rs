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

use archetype::{Archetype, ArchetypeId, ArchetypeInfo, Archetypes};
use component::Component;
use entity::{Entity, EntityGenerator};
use query::TupleIds;
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
        self.entity_generator.create()
    }

    pub fn entity_component_info(&self, entity: Entity) -> Option<&ArchetypeInfo> {
        self.entity_to_archetype
            .get(&entity)
            .and_then(|arch| self.archetypes.get_info(*arch))
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
                let mut arch_info = match self.archetypes.get_info(*arch) {
                    Some(arch_info) => arch_info.clone(),
                    None => Err(EcsError::NonExistingArchetype)?,
                };
                let old_table_id = match self.archetype_to_table.get(arch) {
                    Some(table_id) => *table_id,
                    None => Err(EcsError::NonExistingTable)?,
                };

                arch_info.add_component::<C>()?;

                let new_arch_id = match self.archetypes.get_id(&arch_info) {
                    Some(id) => id,
                    None => {
                        let new_arch_id = self.archetypes.insert(arch_info.clone())?;
                        self.entity_to_archetype.insert(entity, new_arch_id);
                        new_arch_id
                    }
                };

                // Updating current entity with new archetype id
                self.entity_to_archetype.insert(entity, new_arch_id);

                let new_table_id = match self.archetype_to_table.get(&new_arch_id) {
                    Some(new_table_id) => *new_table_id,
                    None => {
                        let new_table_id = self.storage.new_table(&arch_info);
                        self.archetype_to_table.insert(new_arch_id, new_table_id);
                        new_table_id
                    }
                };

                // # Safety
                // Save because tables ids are different
                unsafe {
                    self.storage.transfer_line_with_insertion(
                        old_table_id,
                        new_table_id,
                        entity,
                        component,
                    )?
                };
            }
            None => {
                // The entity does not have an associated compoenet
                let mut arch_info = ArchetypeInfo::default();
                arch_info.add_component::<C>()?;

                let new_arch_id = match self.archetypes.get_id(&arch_info) {
                    Some(id) => id,
                    None => self.archetypes.insert(arch_info.clone())?,
                };

                self.entity_to_archetype.insert(entity, new_arch_id);

                let new_table_id = match self.archetype_to_table.get(&new_arch_id) {
                    Some(new_table_id) => *new_table_id,
                    None => self.storage.new_table(&arch_info),
                };

                self.archetype_to_table.insert(new_arch_id, new_table_id);

                self.storage.add_entity(new_table_id, entity)?;
                self.storage
                    .insert_component(new_table_id, &entity, component)?;
            }
        }
        Ok(())
    }

    /// Removes component from the entity
    /// Returns error if component does not exist
    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Result<(), EcsError> {
        match self.entity_to_archetype.get(&entity) {
            Some(arch) => {
                let mut arch_info = match self.archetypes.get_info(*arch) {
                    Some(arch_info) => arch_info.clone(),
                    None => Err(EcsError::NonExistingArchetype)?,
                };
                let old_table_id = match self.archetype_to_table.get(arch) {
                    Some(table_id) => *table_id,
                    None => Err(EcsError::NonExistingTable)?,
                };

                arch_info.remove_component::<C>()?;

                let new_arch_id = match self.archetypes.get_id(&arch_info) {
                    Some(id) => id,
                    None => {
                        let new_arch_id = self.archetypes.insert(arch_info.clone())?;
                        self.entity_to_archetype.insert(entity, new_arch_id);
                        new_arch_id
                    }
                };

                let new_table_id = match self.archetype_to_table.get(&new_arch_id) {
                    Some(new_table_id) => *new_table_id,
                    None => {
                        let new_table_id = self.storage.new_table(&arch_info);
                        self.archetype_to_table.insert(new_arch_id, new_table_id);
                        new_table_id
                    }
                };

                // # Safety
                // Save because tables ids are different
                unsafe {
                    self.storage
                        .transfer_line_with_deletion(old_table_id, new_table_id, entity)?
                };
            }
            None => Err(EcsError::NonExistingEntity)?,
        }
        Ok(())
    }

    pub fn query<'a, 'b, 'c, const L: usize, C>(
        &'a self,
        archetype: &'b Archetype<'c>,
    ) -> impl Iterator<Item = [&'a (); L]> + '_
    where
        'c: 'a,
        'b: 'c,
        C: TupleIds<L> + 'a,
    {
        let table_id_iter = self
            .archetypes
            .query_ids(archetype)
            .map(|arch_id| self.archetype_to_table[&arch_id]);

        self.storage.query::<L, _, C>(table_id_iter)
    }
}

#[cfg(test)]
mod tests {
    use crate::query::Query;

    use super::*;

    #[test]
    fn ecs_create_entity_and_add_and_remove_component() {
        let mut ecs = Ecs::default();

        let entity = ecs.create();
        ecs.add_component(entity, 1u8).unwrap();
        ecs.add_component(entity, 2u16).unwrap();
        ecs.add_component(entity, 3u32).unwrap();

        let info = ecs.entity_component_info(entity).unwrap();
        assert!(info.has_component::<u8>());
        assert!(info.has_component::<u16>());
        assert!(info.has_component::<u32>());

        ecs.remove_component::<u8>(entity).unwrap();
        ecs.remove_component::<u16>(entity).unwrap();
        ecs.remove_component::<u32>(entity).unwrap();

        let info = ecs.entity_component_info(entity).unwrap();
        assert!(!info.has_component::<u8>());
        assert!(!info.has_component::<u16>());
        assert!(!info.has_component::<u32>());
    }

    #[test]
    fn ecs_multiple_entities() {
        let mut ecs = Ecs::default();

        let entity = ecs.create();
        ecs.add_component(entity, 1u8).unwrap();
        ecs.add_component(entity, 2u16).unwrap();
        ecs.add_component(entity, 3u32).unwrap();

        let info = ecs.entity_component_info(entity).unwrap();
        assert!(info.has_component::<u8>());
        assert!(info.has_component::<u16>());
        assert!(info.has_component::<u32>());

        let entity2 = ecs.create();
        ecs.add_component(entity2, 1u8).unwrap();
        ecs.add_component(entity2, 2u16).unwrap();
        ecs.add_component(entity2, 3u32).unwrap();

        let info = ecs.entity_component_info(entity2).unwrap();
        assert!(info.has_component::<u8>());
        assert!(info.has_component::<u16>());
        assert!(info.has_component::<u32>());
    }

    #[test]
    fn ecs_query() {
        let mut ecs = Ecs::default();

        let entity = ecs.create();
        ecs.add_component(entity, 1u8).unwrap();
        ecs.add_component(entity, 2u16).unwrap();
        ecs.add_component(entity, 3u32).unwrap();

        let entity2 = ecs.create();
        ecs.add_component(entity2, 4u8).unwrap();
        ecs.add_component(entity2, 5u16).unwrap();
        ecs.add_component(entity2, 6u32).unwrap();

        let entity3 = ecs.create();
        ecs.add_component(entity3, 7u8).unwrap();
        ecs.add_component(entity3, 8u16).unwrap();
        ecs.add_component(entity3, 9u64).unwrap();

        let query_ids = Query::<(u8,)>::ids_ref();
        let archetype: Archetype = query_ids.into();
        let query = ecs.query::<1, Query<(u8,)>>(&archetype);
        let mut result = query
            .map(|q| {
                let c1: &u8 = unsafe { std::mem::transmute(q[0]) };
                c1
            })
            .collect::<Vec<_>>();
        result.sort_unstable();
        let expected = [&1, &4, &7];
        assert_eq!(result, expected);

        let query_ids = Query::<(u8, u16)>::ids_ref();
        let archetype: Archetype = query_ids.into();
        let query = ecs.query::<2, Query<(u8, u16)>>(&archetype);
        let mut result = query
            .map(|q| {
                let c1: &u8 = unsafe { std::mem::transmute(q[0]) };
                let c2: &u16 = unsafe { std::mem::transmute(q[1]) };
                (c1, c2)
            })
            .collect::<Vec<_>>();
        result.sort_unstable();
        let expected = [(&1, &2), (&4, &5), (&7, &8)];
        assert_eq!(result, expected);

        let query_ids = Query::<(u8, u16, u32)>::ids_ref();
        let archetype: Archetype = query_ids.into();
        let query = ecs.query::<3, Query<(u8, u16, u32)>>(&archetype);
        let mut result = query
            .map(|q| {
                let c1: &u8 = unsafe { std::mem::transmute(q[0]) };
                let c2: &u16 = unsafe { std::mem::transmute(q[1]) };
                let c3: &u32 = unsafe { std::mem::transmute(q[2]) };
                (c1, c2, c3)
            })
            .collect::<Vec<_>>();
        result.sort_unstable();
        let expected = [(&1, &2, &3), (&4, &5, &6)];
        assert_eq!(result, expected);
    }
}
