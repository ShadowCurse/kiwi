use std::alloc::{Allocator, Global};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};

use crate::component::Component;
use crate::query::QueryCache;
use crate::sparse_set::SparseSet;
use crate::utils::types::{TypeInfo, TypeId};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Adding component dublicate to the archetype")]
    AddingComponentDuplicate,
    #[error("Removing non existing component form the archetype")]
    RemovingNonExistingComponent,
    #[error("Inserting archetype dublicate in component trie")]
    InsertingArchetypeDuplicate,
    #[error("Removing non existing archetype from component trie")]
    RemovingNonExistingArchetype,
    #[error("Trying to access non existing archetype")]
    NonExistingArchetype,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ArchetypeId(usize);

#[derive(Debug, Default, Clone)]
pub struct Archetype<'a> {
    components: Cow<'a, [TypeId]>,
}

impl<'a> Archetype<'a> {
    #[tracing::instrument(skip_all)]
    pub fn iter(&self) -> impl Iterator<Item = &TypeId> {
        self.components.iter()
    }

    #[tracing::instrument(skip_all)]
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

impl<'a> From<&'a [TypeId]> for Archetype<'a> {
    fn from(ids: &'a [TypeId]) -> Self {
        Self {
            components: Cow::from(ids),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ArchetypeInfo {
    components: HashSet<TypeInfo>,
}

impl ArchetypeInfo {
    #[tracing::instrument(skip_all)]
    pub fn archetype(&self) -> Archetype<'static> {
        Archetype {
            components: Cow::Owned(self.as_sorted_vec_of_component_ids()),
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn len(&self) -> usize {
        self.components.len()
    }

    #[tracing::instrument(skip_all)]
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    #[tracing::instrument(skip_all)]
    pub fn add_component<T: Component>(&mut self) -> Result<(), Error> {
        let component_info = TypeInfo::new::<T>();
        match self.components.insert(component_info) {
            true => Ok(()),
            false => Err(Error::AddingComponentDuplicate),
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn has_component<T: Component>(&self) -> bool {
        let component_info = TypeInfo::new::<T>();
        self.components.contains(&component_info)
    }

    #[tracing::instrument(skip_all)]
    pub fn remove_component<T: Component>(&mut self) -> Result<(), Error> {
        let component_info = TypeInfo::new::<T>();
        match self.components.remove(&component_info) {
            true => Ok(()),
            false => Err(Error::RemovingNonExistingComponent),
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn iter(&self) -> impl Iterator<Item = &TypeInfo> {
        self.components.iter()
    }

    #[tracing::instrument(skip_all)]
    pub fn as_sorted_vec_of_component_ids(&self) -> Vec<TypeId> {
        let mut vec = self
            .components
            .iter()
            .map(|info| info.id)
            .collect::<Vec<_>>();
        vec.sort_unstable();
        vec
    }
}

#[derive(Debug, Default)]
pub struct Archetypes {
    archetypes_info: SparseSet<ArchetypeInfo>,
    archetypes_trie: ArchetypesTrie,
}

impl Archetypes {
    #[tracing::instrument(skip_all)]
    pub fn insert(&mut self, archetype_info: ArchetypeInfo) -> Result<ArchetypeId, Error> {
        let arch = archetype_info.archetype();
        let archetype_id = ArchetypeId(self.archetypes_info.insert(archetype_info));
        self.archetypes_trie.insert(arch, archetype_id)?;
        Ok(archetype_id)
    }

    #[tracing::instrument(skip_all)]
    pub fn get_info(&self, archetype_id: ArchetypeId) -> Result<&ArchetypeInfo, Error> {
        self.archetypes_info
            .get(archetype_id.0)
            .ok_or(Error::NonExistingArchetype)
    }

    #[tracing::instrument(skip_all)]
    pub fn get_id(&self, archetype: &ArchetypeInfo) -> Option<ArchetypeId> {
        self.archetypes_trie.search(archetype.archetype())
    }

    #[tracing::instrument(skip_all)]
    pub fn query_ids<'a, 'b>(
        &'a self,
        ids: &'static [TypeId],
    ) -> impl Iterator<Item = ArchetypeId> + 'a
    where
        'b: 'a,
    {
        self.archetypes_trie.query_ids(ids)
    }

    #[tracing::instrument(skip_all)]
    pub fn query_ids_with_cache<'a, 'b>(
        &'a self,
        ids: &'static [TypeId],
        cache: &'a QueryCache,
    ) -> impl Iterator<Item = ArchetypeId> + 'a
    where
        'b: 'a,
    {
        self.archetypes_trie.query_ids_with_cache(ids, cache)
    }
}

#[derive(Debug, Default)]
pub struct ArchetypesTrie {
    root_nodes: Vec<ArchetypeNode>,
    empty_id: Option<ArchetypeId>,
}

impl ArchetypesTrie {
    #[tracing::instrument(skip_all)]
    pub fn insert(&mut self, archetype: Archetype, archetype_id: ArchetypeId) -> Result<(), Error> {
        if archetype.is_empty() {
            self.empty_id = Some(archetype_id);
            Ok(())
        } else {
            Self::recursive_insert(
                &mut self.root_nodes,
                archetype.components.as_ref(),
                0,
                archetype_id,
            )
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn remove(&mut self, archetype: Archetype) -> Result<(), Error> {
        if archetype.is_empty() {
            Ok(())
        } else {
            Self::recursive_remove(&mut self.root_nodes, archetype.components.as_ref(), 0)
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn search(&self, archetype: Archetype) -> Option<ArchetypeId> {
        if archetype.is_empty() {
            self.empty_id
        } else {
            Self::recursive_search(&self.root_nodes, archetype.components.as_ref(), 0)
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn query_ids<'a>(
        &'a self,
        ids: &'static [TypeId],
    ) -> impl Iterator<Item = ArchetypeId> + 'a {
        ArchetypesTrieQueryIterator::new(&self.root_nodes, ids)
    }

    #[tracing::instrument(skip_all)]
    pub fn query_ids_with_cache<'a>(
        &'a self,
        ids: &'static [TypeId],
        cache: &'a QueryCache,
    ) -> impl Iterator<Item = ArchetypeId> + 'a {
        ArchetypesTrieQueryIterator::new_in(&self.root_nodes, ids, &cache.allocator)
    }

    #[tracing::instrument(skip_all)]
    fn recursive_insert(
        nodes: &mut Vec<ArchetypeNode>,
        components: &[TypeId],
        index: usize,
        archetype_id: ArchetypeId,
    ) -> Result<(), Error> {
        match (
            index == components.len() - 1,
            nodes.binary_search_by_key(&components[index], |node| node.component_id),
        ) {
            // Not last component, search next level
            (false, Ok(i)) => Self::recursive_insert(
                &mut nodes[i].following_nodes,
                components,
                index + 1,
                archetype_id,
            ),
            // Last component, node exist
            (true, Ok(i)) => {
                // Node did not have a type
                if nodes[i].archetype_id.is_none() {
                    nodes[i].archetype_id = Some(archetype_id);
                    Ok(())
                } else {
                    Err(Error::InsertingArchetypeDuplicate)
                }
            }
            // Node is not found, inserting one
            (last, Err(i)) => {
                let node = ArchetypeNode::new(components[index]);
                nodes.insert(i, node);
                if last {
                    nodes[i].archetype_id = Some(archetype_id);
                    Ok(())
                } else {
                    Self::recursive_insert(
                        &mut nodes[i].following_nodes,
                        components,
                        index + 1,
                        archetype_id,
                    )
                }
            }
        }
    }

    #[tracing::instrument(skip_all)]
    fn recursive_remove(
        nodes: &mut Vec<ArchetypeNode>,
        components: &[TypeId],
        index: usize,
    ) -> Result<(), Error> {
        match (
            index == components.len() - 1,
            nodes.binary_search_by_key(&components[index], |node| node.component_id),
        ) {
            (false, Ok(i)) => {
                Self::recursive_remove(&mut nodes[i].following_nodes, components, index + 1)
            }
            (true, Ok(i)) => {
                if nodes[i].following_nodes.is_empty() {
                    nodes.remove(i);
                } else {
                    nodes[i].archetype_id = None;
                }
                Ok(())
            }
            (_, Err(_)) => Err(Error::RemovingNonExistingArchetype),
        }
    }

    #[tracing::instrument(skip_all)]
    fn recursive_search(
        nodes: &[ArchetypeNode],
        components: &[TypeId],
        index: usize,
    ) -> Option<ArchetypeId> {
        match (
            index == components.len() - 1,
            nodes.binary_search_by_key(&components[index], |node| node.component_id),
        ) {
            (false, Ok(i)) => {
                Self::recursive_search(&nodes[i].following_nodes, components, index + 1)
            }
            (true, Ok(i)) => nodes[i].archetype_id,
            (_, Err(_)) => None,
        }
    }
}

#[derive(Debug)]
pub struct ArchetypeNode {
    component_id: TypeId,
    archetype_id: Option<ArchetypeId>,
    following_nodes: Vec<ArchetypeNode>,
}

impl ArchetypeNode {
    pub fn new(component: TypeId) -> Self {
        Self {
            component_id: component,
            archetype_id: None,
            following_nodes: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct ArchetypesTrieQueryIteratorEntry<'a> {
    node: &'a ArchetypeNode,
    component_index: usize,
}

#[derive(Debug)]
pub struct ArchetypesTrieQueryIterator<'a, 'b, A: Allocator = Global> {
    entries: VecDeque<ArchetypesTrieQueryIteratorEntry<'a>, A>,
    components_ids: &'b [TypeId],
    found_nodes: VecDeque<&'a ArchetypeNode, A>,
}

impl<'a, 'b> ArchetypesTrieQueryIterator<'a, 'b> {
    #[tracing::instrument(skip_all)]
    pub fn new(initial_nodes: &'a [ArchetypeNode], components_ids: &'static [TypeId]) -> Self {
        let entries = initial_nodes
            .iter()
            .map(|node| ArchetypesTrieQueryIteratorEntry {
                node,
                component_index: 0,
            })
            .collect();
        Self {
            entries,
            components_ids,
            found_nodes: VecDeque::new(),
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn new_in<'alloc, A: Allocator>(
        initial_nodes: &'a [ArchetypeNode],
        components_ids: &'static [TypeId],
        allocator: &'alloc A,
    ) -> ArchetypesTrieQueryIterator<'a, 'b, &'alloc A> {
        let mut entries = VecDeque::new_in(allocator);
        entries.reserve(initial_nodes.len());
        for node in initial_nodes.iter() {
            entries.push_back(ArchetypesTrieQueryIteratorEntry {
                node,
                component_index: 0,
            });
        }
        ArchetypesTrieQueryIterator::<'a, 'b, &'alloc A> {
            entries,
            components_ids,
            found_nodes: VecDeque::new_in(allocator),
        }
    }
}

impl<A: Allocator> Iterator for ArchetypesTrieQueryIterator<'_, '_, A> {
    type Item = ArchetypeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entry) = self.entries.pop_front() {
            match entry
                .node
                .component_id
                .cmp(&self.components_ids[entry.component_index])
            {
                Ordering::Greater => continue,
                Ordering::Less => {
                    for node in entry.node.following_nodes.iter() {
                        self.entries.push_back(ArchetypesTrieQueryIteratorEntry {
                            node,
                            component_index: entry.component_index,
                        });
                    }
                }
                Ordering::Equal => {
                    if entry.component_index == self.components_ids.len() - 1 {
                        for node in entry.node.following_nodes.iter() {
                            self.found_nodes.push_back(node);
                        }
                        match entry.node.archetype_id {
                            Some(archetype) => return Some(archetype),
                            None => continue,
                        }
                    } else {
                        for node in entry.node.following_nodes.iter() {
                            self.entries.push_back(ArchetypesTrieQueryIteratorEntry {
                                node,
                                component_index: entry.component_index + 1,
                            });
                        }
                    }
                }
            }
        }
        while let Some(node) = self.found_nodes.pop_front() {
            for node in node.following_nodes.iter() {
                self.found_nodes.push_back(node);
            }
            match node.archetype_id {
                Some(archetype) => return Some(archetype),
                None => continue,
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::{component::ComponentTuple, impl_component};

    use super::*;

    #[derive(Debug)]
    struct A {}
    #[derive(Debug)]
    struct B {}
    #[derive(Debug)]
    struct C {}
    #[derive(Debug)]
    struct D {}

    impl_component!(A);
    impl_component!(B);
    impl_component!(C);
    impl_component!(D);

    #[test]
    fn archetype_create() {
        let mut arc = ArchetypeInfo::default();
        assert!(arc.is_empty());

        assert!(arc.add_component::<A>().is_ok());
        assert_eq!(arc.len(), 1);
        assert!(arc.add_component::<B>().is_ok());
        assert_eq!(arc.len(), 2);
        assert!(arc.add_component::<C>().is_ok());
        assert_eq!(arc.len(), 3);

        assert!(arc.add_component::<A>().is_err());
        assert!(arc.add_component::<B>().is_err());
        assert!(arc.add_component::<C>().is_err());

        assert!(arc.remove_component::<A>().is_ok());
        assert_eq!(arc.len(), 2);
        assert!(arc.remove_component::<B>().is_ok());
        assert_eq!(arc.len(), 1);
        assert!(arc.remove_component::<C>().is_ok());
        assert_eq!(arc.len(), 0);

        assert!(arc.remove_component::<A>().is_err());
        assert!(arc.remove_component::<B>().is_err());
        assert!(arc.remove_component::<C>().is_err());
    }

    #[test]
    fn component_trie_insert() {
        let mut trie = ArchetypesTrie::default();
        let mut arc = ArchetypeInfo::default();
        let some_arc_id = ArchetypeId(0);

        let _ = arc.add_component::<A>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());

        let _ = arc.add_component::<B>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());

        let _ = arc.add_component::<C>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());

        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());

        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());
        assert!(trie.insert(arc.archetype(), some_arc_id).is_err());
    }

    #[test]
    fn component_trie_search() {
        let mut trie = ArchetypesTrie::default();
        let mut arc = ArchetypeInfo::default();

        let some_arc_id = ArchetypeId(0);
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());
        assert_eq!(trie.search(arc.archetype()), Some(some_arc_id));

        let some_arc_id = ArchetypeId(1);
        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());
        assert_eq!(trie.search(arc.archetype()), Some(some_arc_id));

        let some_arc_id = ArchetypeId(2);
        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());
        assert_eq!(trie.search(arc.archetype()), Some(some_arc_id));

        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<D>();
        assert_eq!(trie.search(arc.archetype()), None);
    }

    #[test]
    fn component_trie_query() {
        let mut trie = ArchetypesTrie::default();
        let mut arc = ArchetypeInfo::default();

        let some_arc_id = ArchetypeId(0);
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());

        let some_arc_id = ArchetypeId(1);
        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());

        let some_arc_id = ArchetypeId(2);
        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());

        let some_arc_id = ArchetypeId(3);
        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());

        let ids = trie.query_ids(&<(&B, &C)>::SORTED_IDS).collect::<HashSet<_>>();
        assert_eq!(
            ids,
            HashSet::from_iter(vec![ArchetypeId(0), ArchetypeId(1)].into_iter())
        );

        let ids = trie.query_ids(&<(&A,)>::SORTED_IDS).collect::<HashSet<_>>();
        assert_eq!(
            ids,
            HashSet::from_iter(vec![ArchetypeId(0), ArchetypeId(2), ArchetypeId(3)].into_iter())
        );
    }
}
