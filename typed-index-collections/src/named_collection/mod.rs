use crate::to1::To1;
use crate::{Index, RawIndex, SemiboundedIndexRange, ValuePerIndexSource};
use std::collections::HashMap;

#[derive(Debug)]
pub struct NamedTo1<InternalIndex: Index, E> {
    store: To1<InternalIndex, E>,
    names: To1<InternalIndex, String>,
    name_to_index: HashMap<String, InternalIndex>,
}

// We cannot derive Default for NamedTo1, because it should implement default even if E does not.
impl<InternalIndex: Index, E> Default for NamedTo1<InternalIndex, E> {
    fn default() -> Self {
        Self::new()
    }
}

// Two collections are equal if they contain the same names mapped to equal entries, independent of
// the order in which the entries were added.
impl<InternalIndex: Index, E: PartialEq> PartialEq for NamedTo1<InternalIndex, E> {
    fn eq(&self, other: &Self) -> bool {
        if self.names == other.names {
            // If the order of names matches, we can do a cheap comparison
            return self.store == other.store;
        }
        // Otherwise, we need to check whether the collections are equal, but names are not ordered
        //  in the same way.
        self.len() == other.len()
            && self.into_iter().all(|(name, entry)| {
                other
                    .entry_by_name(name)
                    .is_some_and(|other_entry| entry == other_entry)
            })
    }
}

impl<InternalIndex: Index, E: Eq> Eq for NamedTo1<InternalIndex, E> {}

impl<InternalIndex: Index, E> NamedTo1<InternalIndex, E> {
    pub fn new() -> Self {
        Self {
            store: To1::new(),
            names: To1::new(),
            name_to_index: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, name: String, entry: E) -> InternalIndex {
        if self.name_to_index.contains_key(&name) {
            panic!("Cannot add a second entry with name `{name}` to this `NamedTo1` collection.")
        }
        let index = self.store.add(entry);
        self.name_to_index.insert(name.clone(), index);
        self.names.add_checked(index, name);
        index
    }

    pub fn get_or_add(&mut self, name: String, entry: E) -> InternalIndex {
        if let Some(entry) = self.name_to_index.get(&name) {
            *entry
        } else {
            let index = self.store.add(entry);
            self.name_to_index.insert(name.clone(), index);
            self.names.add_checked(index, name);
            index
        }
    }

    pub fn map<E2>(self, map: impl FnMut(E) -> E2) -> NamedTo1<InternalIndex, E2> {
        NamedTo1 {
            store: To1::with_entries(self.store.into_iter().map(map).collect::<Vec<_>>()),
            names: self.names,
            name_to_index: self.name_to_index,
        }
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

    pub fn name(&self, index: InternalIndex) -> Option<&str> {
        self.names.get(index).map(String::as_str)
    }

    pub fn names(&self) -> &To1<InternalIndex, String> {
        &self.names
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.name_to_index.contains_key(name)
    }

    pub fn internal_indices(&self) -> SemiboundedIndexRange<InternalIndex> {
        self.store.keys().change_index_type()
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn entries(&self) -> &To1<InternalIndex, E> {
        &self.store
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

impl<'a, InternalIndex: Index, E> IntoIterator for &'a NamedTo1<InternalIndex, E> {
    type Item = (&'a str, &'a E);
    type IntoIter = NamedTo1Iterator<'a, InternalIndex, E>;

    fn into_iter(self) -> Self::IntoIter {
        NamedTo1Iterator {
            named_to_1: self,
            index: InternalIndex::from_raw(InternalIndex::RawType::zero()),
        }
    }
}

pub struct NamedTo1Iterator<'a, InternalIndex: Index, E> {
    named_to_1: &'a NamedTo1<InternalIndex, E>,
    index: InternalIndex,
}

impl<'a, InternalIndex: Index, E> NamedTo1Iterator<'a, InternalIndex, E> {
    pub fn enumerate(self) -> EnumeratingNamedTo1Iterator<'a, InternalIndex, E> {
        EnumeratingNamedTo1Iterator { iterator: self }
    }
}

impl<'a, InternalIndex: Index, E> Iterator for NamedTo1Iterator<'a, InternalIndex, E> {
    type Item = (&'a str, &'a E);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.named_to_1.store.keys().end() {
            let res = Some((
                self.named_to_1.name(self.index).unwrap(),
                &self.named_to_1[self.index],
            ));
            self.index += InternalIndex::RawType::one();
            res
        } else {
            None
        }
    }
}

pub struct EnumeratingNamedTo1Iterator<'a, InternalIndex: Index, E> {
    iterator: NamedTo1Iterator<'a, InternalIndex, E>,
}
impl<'a, InternalIndex: Index, E> Iterator for EnumeratingNamedTo1Iterator<'a, InternalIndex, E> {
    type Item = (InternalIndex, (&'a str, &'a E));

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.iterator.index;
        self.iterator.next().map(|val| (index, val))
    }
}

#[cfg(test)]
mod test {
    use crate as typed_index_collections;
    use crate::NamedTo1;

    crate::index!(EntryIndex);

    fn collection(entries: &[(&str, i32)]) -> NamedTo1<EntryIndex<u32>, i32> {
        let mut collection = NamedTo1::new();
        for &(name, entry) in entries {
            collection.add_entry(name.to_string(), entry);
        }
        collection
    }

    #[test]
    fn equal_same_order() {
        let entries = [("a", 1), ("b", 2), ("c", 3)];
        assert_eq!(collection(&entries), collection(&entries));
    }

    #[test]
    fn equal_different_order() {
        assert_eq!(
            collection(&[("a", 1), ("b", 2), ("c", 3)]),
            collection(&[("c", 3), ("a", 1), ("b", 2)])
        );
    }

    #[test]
    fn equal_empty() {
        assert_eq!(collection(&[]), collection(&[]));
    }

    #[test]
    fn different_entry_same_order() {
        assert_ne!(
            collection(&[("a", 1), ("b", 2)]),
            collection(&[("a", 1), ("b", 3)])
        );
    }

    #[test]
    fn different_entry_different_order() {
        assert_ne!(
            collection(&[("a", 1), ("b", 2)]),
            collection(&[("b", 3), ("a", 1)])
        );
    }

    #[test]
    fn entries_attached_to_swapped_names() {
        assert_ne!(
            collection(&[("a", 1), ("b", 2)]),
            collection(&[("a", 2), ("b", 1)])
        );
        assert_ne!(
            collection(&[("a", 1), ("b", 2)]),
            collection(&[("b", 1), ("a", 2)])
        );
    }

    #[test]
    fn different_names() {
        assert_ne!(
            collection(&[("a", 1), ("b", 2)]),
            collection(&[("a", 1), ("c", 2)])
        );
        assert_ne!(
            collection(&[("a", 1), ("b", 2)]),
            collection(&[("c", 2), ("a", 1)])
        );
    }

    #[test]
    fn different_length() {
        assert_ne!(
            collection(&[("a", 1), ("b", 2)]),
            collection(&[("a", 1), ("b", 2), ("c", 3)])
        );
        assert_ne!(
            collection(&[("a", 1), ("b", 2), ("c", 3)]),
            collection(&[("b", 2), ("a", 1)])
        );
        assert_ne!(collection(&[("a", 1)]), collection(&[]));
    }
}
