use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::archetype::{ArchetypeId, ArchetypeInfo, Archetypes};
use crate::component::{Component, ComponentTuple};
use crate::entity::{Entity, EntityGenerator};
use crate::events::{Event, Events};
use crate::query::QueryCache;
use crate::resources::{Resource, Resources};
use crate::system::{SystemParameter, SystemParameterFetch};
use crate::table::{TableId, TableStorage};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Archetype error: {0}")]
    ArchetypeError(#[from] crate::archetype::Error),
    #[error("Table error: {0}")]
    TableError(#[from] crate::table::Error),
    #[error("Resources error: {0}")]
    Resources(#[from] crate::resources::Error),
    #[error("Archetype has no corresponding table")]
    RogueArchetype,
    #[error("Entity {0} does not exist")]
    NonExistingEntity(Entity),
}

#[derive(Debug, Default)]
pub struct World {
    entity_generator: EntityGenerator,
    archetypes: Archetypes,
    storage: TableStorage,
    resources: Resources,
    /// Mapping of entities to their archetypes
    entity_to_archetype: HashMap<Entity, ArchetypeId>,
    /// Mapping of archetypes to their tables
    archetype_to_table: HashMap<ArchetypeId, TableId>,
}

impl World {
    /// Creates new entity without components
    pub fn create(&mut self) -> Entity {
        self.entity_generator.create()
    }

    pub fn entity_component_info(&self, entity: Entity) -> Option<&ArchetypeInfo> {
        self.entity_to_archetype
            .get(&entity)
            .and_then(|arch| self.archetypes.get_info(*arch).ok())
    }

    /// Adds component to the entity
    /// Returns error if component with the same type is already added to the entity
    #[tracing::instrument(skip(self, entity))]
    pub fn add_component<C: Component>(
        &mut self,
        entity: Entity,
        component: C,
    ) -> Result<(), Error> {
        match self.entity_to_archetype.get(&entity) {
            Some(arch) => {
                let old_table_id = match self.archetype_to_table.get(arch) {
                    Some(table_id) => *table_id,
                    None => Err(Error::RogueArchetype)?,
                };

                let mut arch_info = self.archetypes.get_info(*arch)?.clone();
                arch_info.add_component::<C>()?;

                let new_arch_id = match self.archetypes.get_id(&arch_info) {
                    Some(id) => id,
                    None => self.archetypes.insert(arch_info.clone())?,
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

    /// Updates a component of the entity
    #[tracing::instrument(skip_all)]
    pub fn get_component<C: Component>(&self, entity: Entity) -> Result<&C, Error> {
        match self.entity_to_archetype.get(&entity) {
            Some(arch) => {
                let table_id = match self.archetype_to_table.get(arch) {
                    Some(table_id) => *table_id,
                    None => Err(Error::RogueArchetype)?,
                };

                Ok(self.storage.get_component(table_id, &entity)?)
            }
            None => Err(Error::NonExistingEntity(entity)),
        }
    }

    /// Updates a component of the entity
    #[tracing::instrument(skip_all)]
    pub fn get_component_mut<C: Component>(&mut self, entity: Entity) -> Result<&mut C, Error> {
        match self.entity_to_archetype.get(&entity) {
            Some(arch) => {
                let table_id = match self.archetype_to_table.get(arch) {
                    Some(table_id) => *table_id,
                    None => Err(Error::RogueArchetype)?,
                };

                Ok(self.storage.get_component_mut(table_id, &entity)?)
            }
            None => Err(Error::NonExistingEntity(entity)),
        }
    }

    /// Removes component from the entity
    /// Returns error if component does not exist
    #[tracing::instrument(skip_all)]
    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Result<(), Error> {
        match self.entity_to_archetype.get(&entity) {
            Some(arch) => {
                let old_table_id = match self.archetype_to_table.get(arch) {
                    Some(table_id) => *table_id,
                    None => Err(Error::RogueArchetype)?,
                };

                let mut arch_info = self.archetypes.get_info(*arch)?.clone();
                arch_info.remove_component::<C>()?;

                let new_arch_id = match self.archetypes.get_id(&arch_info) {
                    Some(id) => id,
                    None => self.archetypes.insert(arch_info.clone())?,
                };
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
                    self.storage.transfer_line_with_deletion::<C>(
                        old_table_id,
                        new_table_id,
                        entity,
                    )?
                };
            }
            None => Err(Error::NonExistingEntity(entity))?,
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub fn add_resource<R: Resource>(&mut self, resource: R) {
        self.resources.add(resource)
    }

    #[tracing::instrument(skip_all)]
    pub fn remove_resource<R: Resource>(&mut self) -> Result<(), Error> {
        self.resources.remove::<R>().map_err(Error::Resources)
    }

    #[tracing::instrument(skip_all)]
    pub fn get_resource<R: Resource>(&self) -> Result<&R, Error> {
        self.resources.get::<R>().map_err(Error::Resources)
    }

    #[tracing::instrument(skip_all)]
    pub fn get_resource_mut<R: Resource>(&mut self) -> Result<&mut R, Error> {
        self.resources.get_mut::<R>().map_err(Error::Resources)
    }

    /// # Safety
    /// Save as long as same resource is accessed only once
    #[tracing::instrument(skip_all)]
    pub unsafe fn get_resource_mut_unchecked<R: Resource>(&self) -> Result<&mut R, Error> {
        self.resources
            .get_mut_unchecked::<R>()
            .map_err(Error::Resources)
    }

    #[tracing::instrument(skip_all)]
    pub fn add_event<E: Event>(&mut self) {
        self.resources.add(Events::<E>::default())
    }

    #[tracing::instrument(skip_all)]
    pub fn query<'a, 'b, 'c, CT, const L: usize>(&'a self) -> impl Iterator<Item = CT> + '_
    where
        'c: 'a,
        'b: 'c,
        CT: ComponentTuple<L>,
    {
        let table_id_iter = self
            .archetypes
            .query_ids(&CT::SORTED_IDS)
            .map(|arch_id| self.archetype_to_table[&arch_id]);

        self.storage.query::<_, CT, L>(table_id_iter)
    }

    #[tracing::instrument(skip_all)]
    pub fn query_with_cache<'a, CT, const L: usize>(
        &'a self,
        cache: &'a QueryCache,
    ) -> impl Iterator<Item = CT> + 'a
    where
        CT: ComponentTuple<L>,
    {
        let table_id_iter = self
            .archetypes
            .query_ids_with_cache(&CT::SORTED_IDS, cache)
            .map(|arch_id| self.archetype_to_table[&arch_id]);

        self.storage.query::<_, CT, L>(table_id_iter)
    }
}

#[derive(Debug)]
pub struct WorldRef<'world> {
    world: &'world World,
}

impl Deref for WorldRef<'_> {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        self.world
    }
}

impl SystemParameter for WorldRef<'_> {
    type Fetch = WorldRefFetch;
    // type Cache = ();
}

#[derive(Debug)]
pub struct WorldRefFetch;

impl SystemParameterFetch for WorldRefFetch {
    type Item<'world, 'cache> = WorldRef<'world>;
    type Cache = ();

    fn fetch<'world, 'cache>(
        world: &'world mut World,
        _: &'cache Self::Cache,
    ) -> Self::Item<'world, 'cache> {
        Self::Item { world }
    }
}

#[derive(Debug)]
pub struct WorldRefMut<'world> {
    world: &'world mut World,
}

impl Deref for WorldRefMut<'_> {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        self.world
    }
}

impl DerefMut for WorldRefMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.world
    }
}

impl SystemParameter for WorldRefMut<'_> {
    type Fetch = WorldRefMutFetch;
    // type Cache = ();
}

#[derive(Debug)]
pub struct WorldRefMutFetch;

impl SystemParameterFetch for WorldRefMutFetch {
    type Item<'world, 'cache> = WorldRefMut<'world>;
    type Cache = ();

    fn fetch<'world, 'cache>(
        world: &'world mut World,
        _: &'cache Self::Cache,
    ) -> Self::Item<'world, 'cache> {
        Self::Item { world }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_create_entity_and_add_and_remove_component() {
        let mut ecs = World::default();

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

        ecs.add_component(entity, 1u8).unwrap();
        ecs.add_component(entity, 2u16).unwrap();
        ecs.add_component(entity, 3u32).unwrap();

        let info = ecs.entity_component_info(entity).unwrap();
        assert!(info.has_component::<u8>());
        assert!(info.has_component::<u16>());
        assert!(info.has_component::<u32>());
    }

    #[test]
    fn world_multiple_entities() {
        let mut ecs = World::default();

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
    fn world_query() {
        let mut ecs = World::default();

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

        let query = ecs.query::<(&u8,), 1>();
        let mut result = query
            .map(|q| {
                let c1: &u8 = unsafe { std::mem::transmute(q.0) };
                c1
            })
            .collect::<Vec<_>>();
        result.sort_unstable();
        let expected = [&1, &4, &7];
        assert_eq!(result, expected);

        let query = ecs.query::<(&u8, &u16), 2>();
        let mut result = query
            .map(|q| {
                let c1: &u8 = unsafe { std::mem::transmute(q.0) };
                let c2: &u16 = unsafe { std::mem::transmute(q.1) };
                (c1, c2)
            })
            .collect::<Vec<_>>();
        result.sort_unstable();
        let expected = [(&1, &2), (&4, &5), (&7, &8)];
        assert_eq!(result, expected);

        let query = ecs.query::<(&u8, &u16, &u32), 3>();
        let mut result = query
            .map(|q| {
                let c1: &u8 = unsafe { std::mem::transmute(q.0) };
                let c2: &u16 = unsafe { std::mem::transmute(q.1) };
                let c3: &u32 = unsafe { std::mem::transmute(q.2) };
                (c1, c2, c3)
            })
            .collect::<Vec<_>>();
        result.sort_unstable();
        let expected = [(&1, &2, &3), (&4, &5, &6)];
        assert_eq!(result, expected);
    }
}
