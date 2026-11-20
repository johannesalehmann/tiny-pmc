use crate::{Coalition, QueryEnum};
use std::collections::HashMap;
use std::ops::{Index, Range};

/// This error indicates that a query with the same name already exists in the [`NamedQueries`]
/// structure.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryExists {
    /// The name of the query that already exists
    pub name: String,
    /// The index of the existing query in the [`NamedQueries`] structure
    pub existing_index: usize,
}

/// A collection of [`NamedQuery`], together with a map for efficient name-to-query lookups.
pub struct NamedQueries<I, F, E, C = Coalition> {
    queries: Vec<NamedQuery<I, F, E, C>>,
    name_to_query: HashMap<String, usize>,
}

impl<I, F, E, C> NamedQueries<I, F, E, C> {
    /// Constructs an empty `NamedQueries`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs an empty `NamedQueries`, reserving at least the given `capacity` in its internal
    /// storage fields.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queries: Vec::with_capacity(capacity),
            name_to_query: HashMap::with_capacity(capacity),
        }
    }

    /// Adds a query to the collection. If the query is named and the name already exists, an error
    /// is returned.
    ///
    /// Multiple unnamed queries may be added.
    pub fn add(&mut self, query: NamedQuery<I, F, E, C>) -> Result<(), QueryExists> {
        if let Some(name) = &query.name {
            if let Some(existing_index) = self.name_to_query.get(name).cloned() {
                return Err(QueryExists {
                    name: name.clone(),
                    existing_index,
                });
            }
            let index = self.queries.len();
            let name = name.clone();
            self.queries.push(query);
            self.name_to_query.insert(name, index);
            Ok(())
        } else {
            self.queries.push(query);
            Ok(())
        }
    }

    /// Returns `true` if the collection contains no queries.
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }

    /// Returns the number of queries in the collection.
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Returns a by-reference iterator over this collection's queries
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        (&self).into_iter()
    }

    /// Returns the subset of queries that fulfil the given predicate.
    ///
    /// The predicate has form `pred(i, q)`, where `i` is the index of query `q`.
    pub fn filtered_enumerated<Pred: FnMut(usize, &NamedQuery<I, F, E, C>) -> bool>(
        self,
        mut predicate: Pred,
    ) -> Self {
        let mut queries = Vec::new();
        let mut name_to_query = HashMap::new();
        for (index, query) in self.queries.into_iter().enumerate() {
            if predicate(index, &query) {
                if let Some(name) = &query.name {
                    name_to_query.insert(name.clone(), queries.len());
                }
                queries.push(query);
            }
        }
        Self {
            queries,
            name_to_query,
        }
    }

    /// Returns the subset of queries that fulfil the given predicate.
    pub fn filtered<Pred: FnMut(&NamedQuery<I, F, E, C>) -> bool>(
        self,
        mut predicate: Pred,
    ) -> Self {
        self.filtered_enumerated(|_, q| predicate(q))
    }

    /// Returns the subset of queries that have a name that fulfils the given predicate.
    ///
    /// Queries without names are removed.
    pub fn filtered_by_name<Pred: FnMut(&str) -> bool>(self, mut predicate: Pred) -> Self {
        self.filtered(|q| q.name.is_some() && predicate(&q.name.as_ref().unwrap()))
    }
}

impl<I, F, E, C> From<NamedQueries<I, F, E, C>> for Vec<QueryEnum<I, F, E, C>> {
    fn from(value: NamedQueries<I, F, E, C>) -> Self {
        value.queries.into_iter().map(|q| q.query).collect()
    }
}

impl<I, F, E, C> Default for NamedQueries<I, F, E, C> {
    fn default() -> Self {
        Self {
            queries: Vec::new(),
            name_to_query: HashMap::new(),
        }
    }
}

impl<I, F, E, C> IntoIterator for NamedQueries<I, F, E, C> {
    type Item = NamedQuery<I, F, E, C>;
    type IntoIter = std::vec::IntoIter<NamedQuery<I, F, E, C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.queries.into_iter()
    }
}
impl<'a, I, F, E, C> IntoIterator for &'a NamedQueries<I, F, E, C> {
    type Item = &'a NamedQuery<I, F, E, C>;
    type IntoIter = std::slice::Iter<'a, NamedQuery<I, F, E, C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.queries.iter()
    }
}

impl<I, F, E, C> Index<usize> for NamedQueries<I, F, E, C> {
    type Output = NamedQuery<I, F, E, C>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.queries[index]
    }
}
impl<I, F, E, C> Index<&str> for NamedQueries<I, F, E, C> {
    type Output = NamedQuery<I, F, E, C>;

    fn index(&self, index: &str) -> &Self::Output {
        let index = self.name_to_query[index];
        &self.queries[index]
    }
}

impl<I, F, E, C> Index<Range<usize>> for NamedQueries<I, F, E, C> {
    type Output = [NamedQuery<I, F, E, C>];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.queries[index]
    }
}

/// A query with an optional name.
///
/// Multiple named queries can be collected in a [`NamedQueries`] structure.
pub struct NamedQuery<I, F, E, C = Coalition> {
    // TODO: It would be nice to use an `Identifier` here, but supporting that would require a bit
    //  or re-organisation to make `Identifier` available in this crate
    /// The name of the query or `None` if the query is unnamed
    pub name: Option<String>,
    /// The underlying query
    pub query: QueryEnum<I, F, E, C>,
}

impl<I, F, E, C> NamedQuery<I, F, E, C> {
    /// Constructs a named query with the given `query` and optional name.
    pub fn new(name: Option<String>, query: QueryEnum<I, F, E, C>) -> Self {
        Self { name, query }
    }
    /// Constructs a named query without a name, i.e. with `named_query.name = None`.
    pub fn without_name(query: QueryEnum<I, F, E, C>) -> Self {
        Self::new(None, query)
    }
    /// Constructs a named query with the given name, i.e. with `named_query.name = Some(name)`.
    pub fn with_name(name: String, query: QueryEnum<I, F, E, C>) -> Self {
        Self::new(Some(name), query)
    }
}

impl<I, F, E, C> From<NamedQuery<I, F, E, C>> for QueryEnum<I, F, E, C> {
    fn from(value: NamedQuery<I, F, E, C>) -> Self {
        value.query
    }
}

impl<I, F, E, C> AsRef<QueryEnum<I, F, E, C>> for NamedQuery<I, F, E, C> {
    fn as_ref(&self) -> &QueryEnum<I, F, E, C> {
        &self.query
    }
}

impl<I, F, E, C> AsMut<QueryEnum<I, F, E, C>> for NamedQuery<I, F, E, C> {
    fn as_mut(&mut self) -> &mut QueryEnum<I, F, E, C> {
        &mut self.query
    }
}
