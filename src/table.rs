use std::{any::TypeId, collections::HashMap};

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

    pub fn transfer_components(
        &mut self,
        from: TableId,
        to: TableId,
        entity: Entity,
        archetype: Archetype,
    ) -> Result<(), EcsError> {
        // let from = match self.tables.get_mut(from.0) {
        //     Some(table) => table,
        //     None => Err(EcsError::NonExistingTable)?,
        // };
        // let to = match self.tables.get_mut(to.0) {
        //     Some(table) => table,
        //     None => Err(EcsError::NonExistingTable)?,
        // };
        // for type_id in archetype.iter() {
        //     to.transfer_component(from.get_component(&entity, type_id));
        // }
        // from.invalidate_entity(&entity);
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
    lines: usize,
}

impl Table {
    pub fn clear(&mut self) {
        self.columns.clear();
        self.entities.clear();
        self.lines = 0;
    }

    pub fn invalidate_entity(&mut self, entity: &Entity) {
        todo!()
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

    pub fn transfer_component(&mut self, component: (*const u8, usize)) {
        todo!()
    }

    pub fn get_component(&mut self, entity: &Entity, type_id: &TypeId) -> (*const u8, usize) {
        todo!()
    }

    pub fn insert_entity(&mut self, entity: Entity) {
        self.entities.insert(entity, self.lines);
        self.lines += 1;
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
