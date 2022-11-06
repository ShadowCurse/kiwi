use std::alloc::Layout;
use std::any::TypeId;
use std::collections::HashSet;

use crate::sparse_set::SparseSet;
use crate::EcsError;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ArchetypeId(usize);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ComponentInfo {
    pub id: TypeId,
    pub layout: Layout,
}

impl ComponentInfo {
    pub fn new<T: 'static>() -> Self {
        Self {
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
        }
    }
}

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

    pub fn add_component<T: 'static>(&mut self) -> Result<(), EcsError> {
        let component_info = ComponentInfo::new::<T>();
        match self.components.insert(component_info) {
            true => Ok(()),
            false => Err(EcsError::AddingComponentDuplicate),
        }
    }

    pub fn remove_component<T: 'static>(&mut self) -> Result<(), EcsError> {
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
        let mut vec = self.components.iter().map(|info| info.id).collect::<Vec<_>>();
        vec.sort_unstable();
        vec
    }
}

pub struct Archetypes {
    archetypes: SparseSet<Archetype>,
    archetypes_trie: ArchetypesTrie,
}

impl Archetypes {
    pub fn insert(&mut self, archetype: Archetype) -> Result<ArchetypeId, EcsError>{
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

    fn recursive_insert(
        nodes: &mut Vec<ArchetypeNode>,
        components: &[TypeId],
        index: usize,
        archetype_id: ArchetypeId,
    ) -> Result<(), EcsError> {
        match (
            index == components.len() - 1,
            nodes.binary_search_by_key(&components[index], |node| node.component),
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
            nodes.binary_search_by_key(&components[index], |node| node.component),
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
            nodes.binary_search_by_key(&components[index], |node| node.component),
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
    component: TypeId,
    archetype_id: Option<ArchetypeId>,
    following_nodes: Vec<ArchetypeNode>,
}

impl ArchetypeNode {
    pub fn new(component: TypeId) -> Self {
        Self {
            component,
            archetype_id: None,
            following_nodes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct A {}
    struct B {}
    struct C {}
    struct D {}

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
}
