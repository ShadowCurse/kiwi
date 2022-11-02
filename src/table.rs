use std::{collections::HashMap, any::TypeId};

use crate::{sparse_set::SparseSet, blobvec::BlobVec};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct TableId(usize);

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
pub struct Table {
    columns: HashMap<TypeId, BlobVec>,
}

