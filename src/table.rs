use std::{
    any::TypeId,
    collections::{HashMap, VecDeque},
};

use crate::{
    archetype::Archetype,
    blobvec::BlobVec,
    component::{Component, ComponentInfo},
    entity::Entity,
    sparse_set::SparseSet,
    ArchetypeInfo, EcsError,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct TableId(usize);

#[derive(Debug, Default)]
pub struct TableStorage {
    tables: SparseSet<Table>,
}

impl TableStorage {
    pub fn new_table(&mut self) -> TableId {
        TableId(self.tables.insert(Table::default()))
    }

    pub fn assign_archetype(
        &mut self,
        table_id: TableId,
        archetype_info: &ArchetypeInfo,
    ) -> Result<(), EcsError> {
        match self.tables.get_mut(table_id.0) {
            Some(table) => {
                if !table.columns.is_empty() {
                    Err(EcsError::TableAlreadyAssignedArchetype)
                } else {
                    // TODO maybe there is a better way
                    // Actually archetype should not containt dublicated componens
                    match archetype_info
                        .iter()
                        .map(|type_info| table.register_component(type_info))
                        .collect::<Result<Vec<_>, EcsError>>()
                    {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            table.clear();
                            Err(e)
                        }
                    }
                }
            }
            None => Err(EcsError::TableDoesNotExist),
        }
    }

    /// # Safety
    /// This is safe as long as table ids are different
    pub unsafe fn transfer_components(
        &mut self,
        from: TableId,
        to: TableId,
        entity: Entity,
        archetype: Archetype,
    ) -> Result<(), EcsError> {
        let (from, to) = match self.tables.get_2_mut(from.0, to.0) {
            Some((from, to)) => (from, to),
            None => Err(EcsError::NonExistingTable)?,
        };
        for type_id in archetype.iter() {
            to.copy_component_from(type_id, from.get_component(&entity, type_id))?;
        }
        from.invalidate_entity(&entity);
        Ok(())
    }

    pub fn insert_component<T: Component>(
        &mut self,
        table_id: TableId,
        component: T,
    ) -> Result<(), EcsError> {
        match self.tables.get_mut(table_id.0) {
            Some(table) => table.insert_component(component),
            None => Err(EcsError::TableDoesNotExist),
        }
    }
}

#[derive(Debug, Default)]
pub struct Table {
    columns: HashMap<TypeId, BlobVec>,
    entities: HashMap<Entity, usize>,
    empty_lines: VecDeque<usize>,
}

impl Table {
    pub fn clear(&mut self) {
        self.columns.clear();
        self.entities.clear();
        self.empty_lines.clear();
    }

    pub fn insert_entity(&mut self, entity: Entity) {
        self.entities.insert(entity, self.entities.len());
    }

    pub fn invalidate_entity(&mut self, entity: &Entity) {
        self.empty_lines.push_back(self.entities[entity]);
        self.entities.remove(entity);
    }

    pub fn register_component(&mut self, component_info: &ComponentInfo) -> Result<(), EcsError> {
        match self.columns.contains_key(&component_info.id) {
            false => {
                self.columns
                    .insert(component_info.id, BlobVec::new(component_info.layout));
                Ok(())
            }
            true => Err(EcsError::TableRegisteringDuplicatedComponent),
        }
    }

    pub fn get_component(&self, entity: &Entity, type_id: &TypeId) -> &[u8] {
        unsafe { self.columns[type_id].get_as_byte_slice(self.entities[entity]) }
    }

    pub fn copy_component_from(
        &mut self,
        type_id: &TypeId,
        component: &[u8],
    ) -> Result<(), EcsError> {
        match self.columns.get_mut(type_id) {
            Some(column) => {
                // #Safety
                // We know that slice corresponce to correct type
                unsafe { column.add_from_slice(component) };
                Ok(())
            }
            None => Err(EcsError::TableDoesNotContainComponentColumn),
        }
    }

    pub fn insert_component<T: Component>(&mut self, component: T) -> Result<(), EcsError> {
        let type_id = TypeId::of::<T>();
        match self.columns.get_mut(&type_id) {
            Some(column) => {
                // If column exist for the type
                // then it is safe to add component of this type
                unsafe {
                    column.add(component);
                }
                Ok(())
            }
            None => Err(EcsError::TableDoesNotContainComponentColumn),
        }
    }
}
