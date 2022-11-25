use std::{
    any::TypeId,
    collections::{HashMap, HashSet, VecDeque},
};

use crate::{
    blobvec::BlobVec, component::Component, entity::Entity, sparse_set::SparseSet, ArchetypeInfo,
    EcsError,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct TableId(usize);

#[derive(Debug, Default)]
pub struct TableStorage {
    tables: SparseSet<Table>,
}

impl TableStorage {
    pub fn new_table(&mut self, archetype_info: &ArchetypeInfo) -> TableId {
        let table = Table::new(archetype_info);
        let table_id = self.tables.insert(table);
        TableId(table_id)
    }

    /// # Safety
    /// This is safe as long as table ids are different
    pub unsafe fn transfer_line_with_insertion<T: Component>(
        &mut self,
        from: TableId,
        to: TableId,
        entity: &Entity,
        new_component: T,
    ) -> Result<(), EcsError> {
        let (from, to) = match self.tables.get_2_mut(from.0, to.0) {
            Some((from, to)) => (from, to),
            None => Err(EcsError::NonExistingTable)?,
        };
        to.copy_line_from(from, &entity)?;
        to.insert_component(&entity, new_component)?;
        from.remove_entity(&entity);
        Ok(())
    }

    pub unsafe fn transfer_line_with_deletion(
        &mut self,
        from: TableId,
        to: TableId,
        entity: &Entity,
    ) -> Result<(), EcsError> {
        let (from, to) = match self.tables.get_2_mut(from.0, to.0) {
            Some((from, to)) => (from, to),
            None => Err(EcsError::NonExistingTable)?,
        };
        to.copy_line_from(from, &entity)?;
        from.remove_entity(&entity);
        Ok(())
    }

    pub fn insert_component<T: Component>(
        &mut self,
        table_id: TableId,
        entity: &Entity,
        component: T,
    ) -> Result<(), EcsError> {
        match self.tables.get_mut(table_id.0) {
            Some(table) => table.insert_component(entity, component),
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
    pub fn new(archetype_info: &ArchetypeInfo) -> Self {
        let mut table = Table::default();

        for component_info in archetype_info.iter() {
            table
                .columns
                .insert(component_info.id, BlobVec::new(component_info.layout));
        }
        table
    }

    pub fn intersection(&self, other: &Table) -> Vec<TypeId> {
        self.columns
            .keys()
            .collect::<HashSet<_>>()
            .intersection(&other.columns.keys().collect::<HashSet<_>>())
            .map(|ti| **ti)
            .collect::<Vec<_>>()
    }

    pub fn add_entity(&mut self, entity: Entity) {
        match self.empty_lines.pop_front() {
            Some(line) => {
                self.entities.insert(entity, line);
            }
            None => {
                self.entities.insert(entity, self.entities.len());
                self.allocate_empty_line();
            }
        };
    }

    pub fn remove_entity(&mut self, entity: &Entity) {
        self.empty_lines.push_back(self.entities[entity]);
        self.entities.remove(entity);
    }

    fn get_component_as_slice(&self, entity: &Entity, type_id: &TypeId) -> &[u8] {
        unsafe { self.columns[type_id].get_as_byte_slice(self.entities[entity]) }
    }

    pub fn copy_line_from(&mut self, table: &Table, entity: &Entity) -> Result<(), EcsError> {
        let line = self.entities[entity];
        for type_id in self.intersection(table).iter() {
            self.copy_component_from_slice(
                type_id,
                line,
                table.get_component_as_slice(entity, type_id),
            )?;
        }
        Ok(())
    }

    fn allocate_empty_line(&mut self) {
        for column in self.columns.values_mut() {
            column.push_empty();
        }
    }

    fn copy_component_from_slice(
        &mut self,
        type_id: &TypeId,
        line: usize,
        component: &[u8],
    ) -> Result<(), EcsError> {
        match self.columns.get_mut(type_id) {
            Some(column) => {
                // #Safety
                // We know that slice corresponce to correct type
                unsafe { column.insert_from_slice(line, component) };
                Ok(())
            }
            None => Err(EcsError::TableDoesNotContainComponentColumn),
        }
    }

    pub fn insert_component<T: Component>(
        &mut self,
        entity: &Entity,
        component: T,
    ) -> Result<(), EcsError> {
        let line = self.entities[entity];
        let type_id = TypeId::of::<T>();
        match self.columns.get_mut(&type_id) {
            Some(column) => {
                // If column exist for the type
                // then it is safe to add component of this type
                unsafe {
                    column.insert(line, component);
                }
                Ok(())
            }
            None => Err(EcsError::TableDoesNotContainComponentColumn),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersection() {
        let arc1 = ArchetypeInfo::default();
        arc1.add_component::<u8>().unwrap();
        arc1.add_component::<u16>().unwrap();
        arc1.add_component::<u32>().unwrap();
        let table1 = Table::new(&arc1);

        let arc2 = ArchetypeInfo::default();
        arc1.add_component::<u8>().unwrap();
        arc1.add_component::<u16>().unwrap();
        arc1.add_component::<u64>().unwrap();
        let table2 = Table::new(&arc2);

        assert_eq!(
            table1.intersection(&table2),
            vec![std::any::TypeId::of::<u8>(), std::any::TypeId::of::<u16>()]
        );
    }

    #[test]
    fn transfer_line() {
        let arc1 = ArchetypeInfo::default();
        arc1.add_component::<u8>().unwrap();
        arc1.add_component::<u16>().unwrap();
        arc1.add_component::<u32>().unwrap();
        let table1 = Table::new(&arc1);

        let entity = Entity { id: 1, gen: 0 };

        table1.add_entity(entity);

        table1.insert_component(&entity, 1u8).unwrap();
        table1.insert_component(&entity, 2u16).unwrap();
        table1.insert_component(&entity, 3u32).unwrap();

        let arc2 = ArchetypeInfo::default();
        arc1.add_component::<u8>().unwrap();
        arc1.add_component::<u16>().unwrap();
        arc1.add_component::<u32>().unwrap();
        let table2 = Table::new(&arc2);

        table2.copy_line_from(&table1, &entity).unwrap();
    }
}
