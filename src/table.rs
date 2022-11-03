use std::{any::TypeId, collections::HashMap};

use crate::{
    archetype::ComponentInfo, blobvec::BlobVec, sparse_set::SparseSet, Archetype, EcsError,
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
        archetype: &Archetype,
    ) -> Result<(), EcsError> {
        match self.tables.get_mut(table_id.0) {
            Some(table) => {
                if !table.columns.is_empty() {
                    Err(EcsError::TableAlreadyAssignedArchetype)
                } else {
                    // TODO maybe there is a better way
                    // Actually archetype should not containt dublicated componens
                    match archetype
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

    pub fn transfer_components(&mut self, from: TableId, to: TableId) -> Result<(), EcsError> {
        todo!()
    }

    pub fn insert_component<T: 'static>(
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
}

impl Table {
    pub fn clear(&mut self) {
        self.columns.clear();
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

    pub fn insert_component<T: 'static>(&mut self, component: T) -> Result<(), EcsError> {
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
