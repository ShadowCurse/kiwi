use std::any::TypeId;
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};

use crate::component::{Component, ComponentInfo};
use crate::sparse_set::SparseSet;
use crate::EcsError;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ArchetypeId(usize);

#[derive(Debug, Default)]
pub struct Archetype {
    components: HashSet<ComponentInfo>,
}

impl Archetype {
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

pub struct Archetypes {
    archetypes: SparseSet<Archetype>,
    archetypes_trie: ArchetypesTrie,
}

impl Archetypes {
    pub fn insert(&mut self, archetype: Archetype) -> Result<ArchetypeId, EcsError> {
        let archetype_id = ArchetypeId(self.archetypes.insert(archetype));
        let arc = self.archetypes.get(archetype_id.0).unwrap();
        self.archetypes_trie.insert(arc, archetype_id)?;
        Ok(archetype_id)
    }

    pub fn get(&self, archetype_id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(archetype_id.0)
    }

    pub fn get_mut(&mut self, archetype_id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(archetype_id.0)
    }

    pub fn get_id(&self, archetype: &Archetype) -> Option<ArchetypeId> {
        self.archetypes_trie.search(archetype)
    }
}

#[derive(Debug, Default)]
pub struct ArchetypesTrie {
    root_nodes: Vec<ArchetypeNode>,
}

impl ArchetypesTrie {
    pub fn insert(
        &mut self,
        archetype: &Archetype,
        archetype_id: ArchetypeId,
    ) -> Result<(), EcsError> {
        let components = archetype.as_sorted_vec_of_type_ids();
        Self::recursive_insert(&mut self.root_nodes, &components, 0, archetype_id)
    }

    pub fn remove(&mut self, archetype: &Archetype) -> Result<(), EcsError> {
        let components = archetype.as_sorted_vec_of_type_ids();
        Self::recursive_remove(&mut self.root_nodes, &components, 0)
    }

    pub fn search(&self, archetype: &Archetype) -> Option<ArchetypeId> {
        let components = archetype.as_sorted_vec_of_type_ids();
        Self::recursive_search(&self.root_nodes, &components, 0)
    }

    pub fn query(&self, sub_suquence: &Archetype) -> impl Iterator<Item = ArchetypeId> + '_ {
        ArchetypesTrieQueryIterator::new(&self.root_nodes, sub_suquence.as_sorted_vec_of_type_ids())
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

struct ArchetypesTrieQueryIterator<'a> {
    entries: VecDeque<ArchetypesTrieQueryIteratorEntry<'a>>,
    components: Vec<TypeId>,
    found_nodes: VecDeque<&'a ArchetypeNode>,
}

impl<'a> ArchetypesTrieQueryIterator<'a> {
    pub fn new(initial_nodes: &'a [ArchetypeNode], components: Vec<TypeId>) -> Self {
        let nodes = initial_nodes
            .iter()
            .map(|node| ArchetypesTrieQueryIteratorEntry {
                node,
                component_index: 0,
            })
            .collect();
        Self {
            entries: nodes,
            components,
            found_nodes: VecDeque::new(),
        }
    }
}

impl Iterator for ArchetypesTrieQueryIterator<'_> {
    type Item = ArchetypeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.found_nodes.pop_front() {
            for node in node.following_nodes.iter() {
                self.found_nodes.push_back(node);
            }
            match node.archetype_id {
                Some(archetype) => return Some(archetype),
                None => continue,
            }
        }
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
                        // return every node starting from this root
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
        let mut arc = Archetype::default();
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
        let mut arc = Archetype::default();
        let some_arc_id = ArchetypeId(0);

        let _ = arc.add_component::<A>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());
        println!("-----------------------");
        println!("{:#?}", trie);
        println!("-----------------------");

        let _ = arc.add_component::<B>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());
        println!("-----------------------");
        println!("{:#?}", trie);
        println!("-----------------------");

        let _ = arc.add_component::<C>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());
        println!("-----------------------");
        println!("{:#?}", trie);
        println!("-----------------------");

        let mut arc = Archetype::default();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());
        println!("-----------------------");
        println!("{:#?}", trie);
        println!("-----------------------");

        let mut arc = Archetype::default();
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());
        assert!(trie.insert(&arc, some_arc_id).is_err());
        println!("-----------------------");
        println!("{:#?}", trie);
        println!("-----------------------");
    }

    #[test]
    fn component_trie_search() {
        let mut trie = ArchetypesTrie::default();
        let mut arc = Archetype::default();

        let some_arc_id = ArchetypeId(0);
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());
        assert_eq!(trie.search(&arc), Some(some_arc_id));

        let some_arc_id = ArchetypeId(1);
        let mut arc = Archetype::default();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());
        assert_eq!(trie.search(&arc), Some(some_arc_id));

        let some_arc_id = ArchetypeId(2);
        let mut arc = Archetype::default();
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());
        assert_eq!(trie.search(&arc), Some(some_arc_id));

        let mut arc = Archetype::default();
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<D>();
        assert_eq!(trie.search(&arc), None);
    }

    #[test]
    fn component_trie_query() {
        let mut trie = ArchetypesTrie::default();
        let mut arc = Archetype::default();

        let some_arc_id = ArchetypeId(0);
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());

        let some_arc_id = ArchetypeId(1);
        let mut arc = Archetype::default();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());

        let some_arc_id = ArchetypeId(2);
        let mut arc = Archetype::default();
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<C>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());

        let some_arc_id = ArchetypeId(3);
        let mut arc = Archetype::default();
        let _ = arc.add_component::<A>();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<D>();
        assert!(trie.insert(&arc, some_arc_id).is_ok());

        let mut arc = Archetype::default();
        let _ = arc.add_component::<B>();
        let _ = arc.add_component::<C>();
        let ids = trie.query(&arc).collect::<HashSet<_>>();
        assert_eq!(
            ids,
            HashSet::from_iter(vec![ArchetypeId(0), ArchetypeId(1)].into_iter())
        );

        let mut arc = Archetype::default();
        let _ = arc.add_component::<A>();
        let ids = trie.query(&arc).collect::<HashSet<_>>();
        assert_eq!(
            ids,
            HashSet::from_iter(vec![ArchetypeId(0), ArchetypeId(2), ArchetypeId(3)].into_iter())
        );
    }
}
