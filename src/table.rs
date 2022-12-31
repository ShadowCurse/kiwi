use std::{
    any::TypeId,
    collections::{hash_map::Values, HashMap, HashSet, VecDeque},
    marker::PhantomData,
};

use crate::{
    blobvec::BlobVec,
    component::{Component, ComponentTuple},
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
    pub fn new_table(&mut self, archetype_info: &ArchetypeInfo) -> TableId {
        let table = Table::new(archetype_info);
        let table_id = self.tables.insert(table);
        TableId(table_id)
    }

    pub fn add_entity(&mut self, table_id: TableId, entity: Entity) -> Result<(), EcsError> {
        match self.tables.get_mut(table_id.0) {
            Some(table) => {
                table.add_entity(entity);
                Ok(())
            }
            None => Err(EcsError::TableDoesNotExist),
        }
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

    /// # Safety
    /// This is safe as long as table ids are different
    pub unsafe fn transfer_line_with_insertion<T: Component>(
        &mut self,
        from: TableId,
        to: TableId,
        entity: Entity,
        new_component: T,
    ) -> Result<(), EcsError> {
        let (from, to) = match self.tables.get_2_mut(from.0, to.0) {
            Some((from, to)) => (from, to),
            None => Err(EcsError::NonExistingTable)?,
        };
        to.add_entity(entity);
        to.copy_line_from(from, &entity)?;
        to.insert_component(&entity, new_component)?;
        from.remove_entity(&entity);
        Ok(())
    }

    pub unsafe fn transfer_line_with_deletion(
        &mut self,
        from: TableId,
        to: TableId,
        entity: Entity,
    ) -> Result<(), EcsError> {
        let (from, to) = match self.tables.get_2_mut(from.0, to.0) {
            Some((from, to)) => (from, to),
            None => Err(EcsError::NonExistingTable)?,
        };
        to.add_entity(entity);
        to.copy_line_from(from, &entity)?;
        from.remove_entity(&entity);
        Ok(())
    }

    pub fn query<const L: usize, I, C>(&self, table_id_iter: I) -> TableStorageIterator<'_, L, I, C>
    where
        I: Iterator<Item = TableId>,
        C: ComponentTuple<L>,
    {
        TableStorageIterator {
            storage: self,
            table_id_iter,
            component_iter: None,
            phantom: PhantomData,
        }
    }

    fn get_table(&self, table_id: TableId) -> Option<&Table> {
        self.tables.get(table_id.0)
    }
}

pub struct TableStorageIterator<'a, const L: usize, I, C>
where
    I: Iterator<Item = TableId>,
    C: ComponentTuple<L>,
{
    storage: &'a TableStorage,
    table_id_iter: I,
    component_iter: Option<TableIterator<'a, L>>,
    phantom: PhantomData<C>,
}

impl<'a, const L: usize, I, C> Iterator for TableStorageIterator<'a, L, I, C>
where
    I: Iterator<Item = TableId>,
    C: ComponentTuple<L>,
{
    type Item = <TableIterator<'a, L> as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.component_iter {
            Some(ref mut component_iter) => match component_iter.next() {
                Some(component) => Some(component),
                None => {
                    self.component_iter = None;
                    self.next()
                }
            },
            None => match self.table_id_iter.next() {
                Some(table_id) => {
                    let table = self.storage.get_table(table_id).unwrap();
                    self.component_iter = Some(table.component_iter::<L, C>());
                    self.next()
                }
                None => None,
            },
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

    pub fn get_component<T: Component>(&self, entity: &Entity) -> Option<&T> {
        let line = self.entities[entity];
        let type_id = TypeId::of::<T>();
        self.columns
            .get(&type_id)
            .map(|column| unsafe { column.get(line) })
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: &Entity) -> Option<&mut T> {
        let line = self.entities[entity];
        let type_id = TypeId::of::<T>();
        self.columns
            .get_mut(&type_id)
            .map(|column| unsafe { column.get_mut(line) })
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

    pub fn component_iter<const L: usize, C>(&self) -> TableIterator<'_, L>
    where
        C: ComponentTuple<L>,
    {
        let columns = C::ids().map(|id| &self.columns[&id]);
        TableIterator {
            columns,
            entities: self.entities.values(),
        }
    }
}

#[derive(Debug)]
pub struct TableIterator<'a, const L: usize> {
    columns: [&'a BlobVec; L],
    entities: Values<'a, Entity, usize>,
}

impl<'a, const L: usize> Iterator for TableIterator<'a, L> {
    type Item = [&'a (); L];

    fn next(&mut self) -> Option<Self::Item> {
        self.entities.next().map(|line| {
            self.columns
                .map(|column| unsafe { column.get_erased_ref(*line) })
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn table_intersection() {
        let mut arc1 = ArchetypeInfo::default();
        arc1.add_component::<u8>().unwrap();
        arc1.add_component::<u16>().unwrap();
        arc1.add_component::<u32>().unwrap();
        let table1 = Table::new(&arc1);

        let mut arc2 = ArchetypeInfo::default();
        arc2.add_component::<u8>().unwrap();
        arc2.add_component::<u16>().unwrap();
        arc2.add_component::<u64>().unwrap();
        let table2 = Table::new(&arc2);

        let mut intersection = table1.intersection(&table2);
        intersection.sort_unstable();

        let mut expected = vec![std::any::TypeId::of::<u8>(), std::any::TypeId::of::<u16>()];
        expected.sort_unstable();
        assert_eq!(intersection, expected);
    }

    #[test]
    fn table_transfer_line() {
        let mut arc1 = ArchetypeInfo::default();
        arc1.add_component::<u8>().unwrap();
        arc1.add_component::<u16>().unwrap();
        arc1.add_component::<u32>().unwrap();
        let mut table1 = Table::new(&arc1);

        let entity = Entity::from_raw(1, 0);

        table1.add_entity(entity);

        table1.insert_component(&entity, 1u8).unwrap();
        table1.insert_component(&entity, 2u16).unwrap();
        table1.insert_component(&entity, 3u32).unwrap();

        let mut arc2 = ArchetypeInfo::default();
        arc2.add_component::<u8>().unwrap();
        arc2.add_component::<u16>().unwrap();
        arc2.add_component::<u32>().unwrap();
        let mut table2 = Table::new(&arc2);

        table2.add_entity(entity);
        table2.copy_line_from(&table1, &entity).unwrap();

        assert_eq!(
            table1.get_component::<u8>(&entity),
            table2.get_component::<u8>(&entity)
        );
        assert_eq!(
            table1.get_component::<u16>(&entity),
            table2.get_component::<u16>(&entity)
        );
        assert_eq!(
            table1.get_component::<u32>(&entity),
            table2.get_component::<u32>(&entity)
        );
    }

    #[test]
    fn table_storage_transfer_insert() {
        let mut arc1 = ArchetypeInfo::default();
        arc1.add_component::<u8>().unwrap();
        arc1.add_component::<u16>().unwrap();
        arc1.add_component::<u32>().unwrap();

        let mut table_storage = TableStorage::default();
        let table_id_1 = table_storage.new_table(&arc1);

        let entity = Entity::from_raw(1, 0);
        table_storage.add_entity(table_id_1, entity).unwrap();

        table_storage
            .insert_component(table_id_1, &entity, 1u8)
            .unwrap();
        table_storage
            .insert_component(table_id_1, &entity, 2u16)
            .unwrap();
        table_storage
            .insert_component(table_id_1, &entity, 3u32)
            .unwrap();

        let mut arc2 = ArchetypeInfo::default();
        arc2.add_component::<u8>().unwrap();
        arc2.add_component::<u16>().unwrap();
        arc2.add_component::<u32>().unwrap();
        arc2.add_component::<u64>().unwrap();
        let table_id_2 = table_storage.new_table(&arc2);

        unsafe {
            table_storage
                .transfer_line_with_insertion(table_id_1, table_id_2, entity, 4u64)
                .unwrap()
        };

        assert_eq!(
            table_storage
                .tables
                .get(table_id_1.0)
                .unwrap()
                .entities
                .len(),
            0
        );
        assert_eq!(
            table_storage.tables.get(table_id_1.0).unwrap().empty_lines,
            vec![0]
        );

        assert_eq!(
            table_storage
                .tables
                .get(table_id_2.0)
                .unwrap()
                .entities
                .len(),
            1
        );
        assert_eq!(
            table_storage
                .tables
                .get(table_id_2.0)
                .unwrap()
                .get_component::<u8>(&entity)
                .unwrap(),
            &1u8
        );
        assert_eq!(
            table_storage
                .tables
                .get(table_id_2.0)
                .unwrap()
                .get_component::<u16>(&entity)
                .unwrap(),
            &2u16
        );
        assert_eq!(
            table_storage
                .tables
                .get(table_id_2.0)
                .unwrap()
                .get_component::<u32>(&entity)
                .unwrap(),
            &3u32
        );
        assert_eq!(
            table_storage
                .tables
                .get(table_id_2.0)
                .unwrap()
                .get_component::<u64>(&entity)
                .unwrap(),
            &4u64
        );
    }

    #[test]
    fn table_storage_transfer_delete() {
        let mut arc1 = ArchetypeInfo::default();
        arc1.add_component::<u8>().unwrap();
        arc1.add_component::<u16>().unwrap();
        arc1.add_component::<u32>().unwrap();

        let mut table_storage = TableStorage::default();
        let table_id_1 = table_storage.new_table(&arc1);

        let entity = Entity::from_raw(1, 0);
        table_storage.add_entity(table_id_1, entity).unwrap();

        table_storage
            .insert_component(table_id_1, &entity, 1u8)
            .unwrap();
        table_storage
            .insert_component(table_id_1, &entity, 2u16)
            .unwrap();
        table_storage
            .insert_component(table_id_1, &entity, 3u32)
            .unwrap();

        let mut arc2 = ArchetypeInfo::default();
        arc2.add_component::<u8>().unwrap();
        arc2.add_component::<u16>().unwrap();
        let table_id_2 = table_storage.new_table(&arc2);

        unsafe {
            table_storage
                .transfer_line_with_deletion(table_id_1, table_id_2, entity)
                .unwrap()
        };

        assert_eq!(
            table_storage
                .tables
                .get(table_id_1.0)
                .unwrap()
                .entities
                .len(),
            0
        );
        assert_eq!(
            table_storage.tables.get(table_id_1.0).unwrap().empty_lines,
            vec![0]
        );

        assert_eq!(
            table_storage
                .tables
                .get(table_id_2.0)
                .unwrap()
                .entities
                .len(),
            1
        );
        assert_eq!(
            table_storage
                .tables
                .get(table_id_2.0)
                .unwrap()
                .get_component::<u8>(&entity)
                .unwrap(),
            &1u8
        );
        assert_eq!(
            table_storage
                .tables
                .get(table_id_2.0)
                .unwrap()
                .get_component::<u16>(&entity)
                .unwrap(),
            &2u16
        );
    }
}
