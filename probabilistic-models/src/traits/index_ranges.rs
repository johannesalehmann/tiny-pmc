use typed_index_collections::{Index, RawIndex};

macro_rules! entity_collection {
    ($name: ident, $index: ident) => {
        paste::paste! {
            #[derive(Copy, Clone, PartialEq, Eq, Debug)]
            pub struct $name<$index: Index> {
                size: $index,
            }

            impl<$index: Index> $name<$index> {
                pub fn new(size: $index) -> Self {
                    Self { size }
                }

                pub fn from_usize(size: usize) -> Self {
                    Self { size: $index::from_raw($index::RawType::from_usize(size)) }
                }

                pub fn len(&self) -> usize {
                    self.size.raw().as_usize()
                }
            }

            impl<$index: Index> IntoIterator for $name<$index> {
                type Item = $index;
                type IntoIter = [<$name Iterator>]<$index>;

                fn into_iter(self) -> Self::IntoIter {
                    [<$name Iterator>] {
                        next: $index::from_raw($index::RawType::zero()),
                        end: self.size,
                    }
                }
            }

            pub struct [<$name Iterator>]<$index: Index> {
                next: $index,
                end: $index,
            }

            impl<$index: Index> Iterator for [<$name Iterator>]<$index> {
                type Item = $index;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.next.raw() < self.end.raw() {
                        let res = self.next.clone();
                        self.next += $index::RawType::one();
                        Some(res)
                    } else {
                        None
                    }
                }

                fn size_hint(&self) -> (usize, Option<usize>) {
                    let length = self.end.raw().as_usize() - self.next.raw().as_usize();
                    (length, Some(length))
                }

                fn last(mut self) -> Option<Self::Item>
                where
                    Self: Sized,
                {
                    if self.next.raw() < self.end.raw() {
                        self.end = self.end - $index::RawType::one();
                        Some(self.end)
                    } else {
                        None
                    }
                }
            }
        }
    };
}

entity_collection!(States, StateIdx);
entity_collection!(Choices, ChoiceIdx);
entity_collection!(Branches, BranchIdx);

macro_rules! entity_range {
    ($name: ident, $index: ident) => {
        paste::paste! {
            #[derive(Copy, Clone, PartialEq, Eq, Debug)]
            pub struct $name<$index: Index> {
                start: $index,
                end: $index,
            }

            impl<$index: Index> $name<$index> {
                pub fn new(start: $index, end: $index) -> Self {
                    Self {
                        start, end
                    }
                }

                pub fn from_usize(start: usize, end: usize) -> Self {
                    Self {
                        start: $index::from_raw($index::RawType::from_usize(start)),
                        end: $index::from_raw($index::RawType::from_usize(start))
                    }
                }

                pub fn with_single_entry(entry: $index) -> Self {
                    Self {
                        start: entry,
                        end: entry + $index::RawType::one()
                    }
                }

                pub fn with_single_entry_usize(entry: usize) -> Self {
                    Self {
                        start: $index::from_raw($index::RawType::from_usize(entry)),
                        end: $index::from_raw($index::RawType::from_usize(entry + 1))
                    }
                }
            }


            impl<$index: Index> IntoIterator for $name<$index> {
                type Item = $index;
                type IntoIter = [<$name Iterator>]<$index>;

                fn into_iter(self) -> Self::IntoIter {
                    [<$name Iterator>] {
                        next: self.start,
                        end: self.end,
                    }
                }
            }

            pub struct [<$name Iterator>]<$index: Index> {
                next: $index,
                end: $index,
            }

            impl<$index: Index> Iterator for [<$name Iterator>]<$index> {
                type Item = $index;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.next.raw() < self.end.raw() {
                        let res = self.next.clone();
                        self.next += $index::RawType::one();
                        Some(res)
                    } else {
                        None
                    }
                }

                fn size_hint(&self) -> (usize, Option<usize>) {
                    let length = self.end.raw().as_usize() - self.next.raw().as_usize();
                    (length, Some(length))
                }

                fn last(mut self) -> Option<Self::Item>
                where
                    Self: Sized,
                {
                    if self.next.raw() < self.end.raw() {
                        self.end = self.end - $index::RawType::one();
                        Some(self.end)
                    } else {
                        None
                    }
                }
            }
        }
    };
}

entity_range!(ChoiceRange, ChoiceIndex);
entity_range!(BranchRange, BranchIndex);
