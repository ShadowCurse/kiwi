use crate::sparse_set::SparseSet;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct TableId(usize);

pub struct TableStorage {
    tables: SparseSet<Table>,
}

impl TableStorage {
    pub fn create(&mut self) -> TableId {
        todo!()
    }

    pub fn transfer_components(&mut self, from: TableId, to: TableId) -> TableId {
        todo!()
    }

    pub fn insert_component<T>(&mut self, table_id: TableId, component: T) -> TableId {
        todo!()
    }
}

pub struct Table {}

