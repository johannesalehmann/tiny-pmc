use crate::Index;
use crate::to1::To1;
use std::collections::HashMap;

#[derive(Default)]
pub struct NamedTo1<InternalIndex: Index, E> {
    store: To1<InternalIndex, E>,
    name_to_index: HashMap<String, InternalIndex>,
}

impl<InternalIndex: Index, E> NamedTo1<InternalIndex, E> {
    pub fn add_entry(&mut self, name: String, entry: E) -> InternalIndex {
        if self.name_to_index.contains_key(&name) {
            panic!("Cannot add a second entry with name `{name}` to this `NamedTo1` collection.")
        }
        let index = self.store.add_unchecked(entry);
        self.name_to_index.insert(name, index);
        index
    }

    pub fn get(&self, index: InternalIndex) -> Option<&E> {
        self.store.get(index)
    }

    pub fn get_mut(&mut self, index: InternalIndex) -> Option<&mut E> {
        self.store.get_mut(index)
    }

    pub fn entry_by_name(&self, name: &str) -> Option<&E> {
        let index = self.name_to_index.get(name)?;
        self.store.get(*index)
    }

    pub fn entry_by_name_mut(&mut self, name: &str) -> Option<&mut E> {
        let index = self.name_to_index.get(name)?;
        self.store.get_mut(*index)
    }

    pub fn index_by_name(&self, name: &str) -> Option<InternalIndex> {
        self.name_to_index.get(name).cloned()
    }
}

impl<InternalIndex: Index, E> std::ops::Index<InternalIndex> for NamedTo1<InternalIndex, E> {
    type Output = E;

    fn index(&self, index: InternalIndex) -> &Self::Output {
        &self.store[index]
    }
}

impl<InternalIndex: Index, E> std::ops::IndexMut<InternalIndex> for NamedTo1<InternalIndex, E> {
    fn index_mut(&mut self, index: InternalIndex) -> &mut Self::Output {
        &mut self.store[index]
    }
}

impl<InternalIndex: Index, E> std::ops::Index<&str> for NamedTo1<InternalIndex, E> {
    type Output = E;

    fn index(&self, name: &str) -> &Self::Output {
        let index = self
            .name_to_index
            .get(name)
            .expect("This `NamedTo1` collection contains no  entry with name `{name}`.");
        &self.store[*index]
    }
}

impl<InternalIndex: Index, E> std::ops::IndexMut<&str> for NamedTo1<InternalIndex, E> {
    fn index_mut(&mut self, name: &str) -> &mut Self::Output {
        let index = self
            .name_to_index
            .get(name)
            .expect("This `NamedTo1` collection contains no  entry with name `{name}`.");
        &mut self.store[*index]
    }
}
