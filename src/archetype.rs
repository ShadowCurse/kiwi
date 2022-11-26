use std::any::TypeId;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};

use crate::component::{Component, ComponentInfo};
use crate::sparse_set::SparseSet;
use crate::EcsError;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ArchetypeId(usize);

#[derive(Debug, Default, Clone)]
pub struct Archetype<'a> {
    components: Cow<'a, [TypeId]>,
}

impl<'a> Archetype<'a> {
    pub fn iter(&self) -> impl Iterator<Item = &TypeId> {
        self.components.iter()
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
    components: HashSet<ComponentInfo>,
}

impl ArchetypeInfo {
    pub fn archetype(&self) -> Archetype<'static> {
        Archetype {
            components: Cow::Owned(self.as_sorted_vec_of_type_ids()),
        }
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub fn add_component<T: Component>(&mut self) -> Result<(), EcsError> {
        let component_info = ComponentInfo::new::<T>();
        match self.components.insert(component_info) {
            true => Ok(()),
            false => Err(EcsError::AddingComponentDuplicate),
        }
    }

    pub fn remove_component<T: Component>(&mut self) -> Result<(), EcsError> {
        let component_info = ComponentInfo::new::<T>();
        match self.components.remove(&component_info) {
            true => Ok(()),
            false => Err(EcsError::RemovingNonExistingComponent),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ComponentInfo> {
        self.components.iter()
    }

    pub fn as_sorted_vec_of_type_ids(&self) -> Vec<TypeId> {
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
    pub fn insert(&mut self, archetype_info: ArchetypeInfo) -> Result<ArchetypeId, EcsError> {
        let arch = archetype_info.archetype();
        let archetype_id = ArchetypeId(self.archetypes_info.insert(archetype_info));
        self.archetypes_trie.insert(arch, archetype_id)?;
        Ok(archetype_id)
    }

    pub fn get_info(&self, archetype_id: ArchetypeId) -> Option<&ArchetypeInfo> {
        self.archetypes_info.get(archetype_id.0)
    }

    pub fn get_id(&self, archetype: &ArchetypeInfo) -> Option<ArchetypeId> {
        self.archetypes_trie.search(archetype.archetype())
    }
}

#[derive(Debug, Default)]
pub struct ArchetypesTrie {
    root_nodes: Vec<ArchetypeNode>,
}

impl ArchetypesTrie {
    pub fn insert(
        &mut self,
        archetype: Archetype,
        archetype_id: ArchetypeId,
    ) -> Result<(), EcsError> {
        Self::recursive_insert(
            &mut self.root_nodes,
            archetype.components.as_ref(),
            0,
            archetype_id,
        )
    }

    pub fn remove(&mut self, archetype: Archetype) -> Result<(), EcsError> {
        Self::recursive_remove(&mut self.root_nodes, archetype.components.as_ref(), 0)
    }

    pub fn search(&self, archetype: Archetype) -> Option<ArchetypeId> {
        Self::recursive_search(&self.root_nodes, archetype.components.as_ref(), 0)
    }

    pub fn query<'a, 'b>(
        &'a self,
        sub_suquence: &'a Archetype<'b>,
    ) -> impl Iterator<Item = ArchetypeId> + 'a
    where
        'b: 'a,
    {
        ArchetypesTrieQueryIterator::new(&self.root_nodes, sub_suquence.components.as_ref())
    }

    fn recursive_insert(
        nodes: &mut Vec<ArchetypeNode>,
        components: &[TypeId],
        index: usize,
        archetype_id: ArchetypeId,
    ) -> Result<(), EcsError> {
        match (
            index == components.len() - 1,
            nodes.binary_search_by_key(&components[index], |node| node.component_id),
        ) {
            (false, Ok(i)) => Self::recursive_insert(
                &mut nodes[i].following_nodes,
                components,
                index + 1,
                archetype_id,
            ),
            (true, Ok(_)) => Err(EcsError::InsertingArchetypeDuplicate),
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

    fn recursive_remove(
        nodes: &mut Vec<ArchetypeNode>,
        components: &[TypeId],
        index: usize,
    ) -> Result<(), EcsError> {
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
            (_, Err(_)) => Err(EcsError::RemovingNonExistingArchetype),
        }
    }

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
struct ArchetypesTrieQueryIterator<'a, 'b> {
    entries: VecDeque<ArchetypesTrieQueryIteratorEntry<'a>>,
    components: &'b [TypeId],
    found_nodes: VecDeque<&'a ArchetypeNode>,
}

impl<'a, 'b> ArchetypesTrieQueryIterator<'a, 'b> {
    pub fn new(initial_nodes: &'a [ArchetypeNode], components: &'b [TypeId]) -> Self {
        let entries = initial_nodes
            .iter()
            .map(|node| ArchetypesTrieQueryIteratorEntry {
                node,
                component_index: 0,
            })
            .collect();
        Self {
            entries,
            components,
            found_nodes: VecDeque::new(),
        }
    }
}

impl Iterator for ArchetypesTrieQueryIterator<'_, '_> {
    type Item = ArchetypeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entry) = self.entries.pop_front() {
            match entry
                .node
                .component_id
                .cmp(&self.components[entry.component_index])
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
                    if entry.component_index == self.components.len() - 1 {
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
    use super::*;

    struct A {}
    struct B {}
    struct C {}
    struct D {}

    impl Component for A {}
    impl Component for B {}
    impl Component for C {}
    impl Component for D {}

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
        println!("-----------------------");
        println!("{:#?}", trie);
        println!("-----------------------");

        let _ = arc.add_component::<B>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());
        println!("-----------------------");
        println!("{:#?}", trie);
        println!("-----------------------");

        let _ = arc.add_component::<C>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());
        println!("-----------------------");
        println!("{:#?}", trie);
        println!("-----------------------");

        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());
        println!("-----------------------");
        println!("{:#?}", trie);
        println!("-----------------------");

        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(arc.archetype(), some_arc_id).is_ok());
        assert!(trie.insert(arc.archetype(), some_arc_id).is_err());
        println!("-----------------------");
        println!("{:#?}", trie);
        println!("-----------------------");
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

        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        let ids = trie.query(&arc.archetype()).collect::<HashSet<_>>();
        assert_eq!(
            ids,
            HashSet::from_iter(vec![ArchetypeId(0), ArchetypeId(1)].into_iter())
        );

        let mut arc = ArchetypeInfo::default();
        let _ = arc.add_component::<A>();
        let ids = trie.query(&arc.archetype()).collect::<HashSet<_>>();
        assert_eq!(
            ids,
            HashSet::from_iter(vec![ArchetypeId(0), ArchetypeId(2), ArchetypeId(3)].into_iter())
        );
    }
}
