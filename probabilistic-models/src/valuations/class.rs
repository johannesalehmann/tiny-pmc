use crate::{RawIndex, ValuationClassEntryIndex};
use std::collections::HashMap;
use std::ops::Range;
use typed_index_collections::To1;

pub struct ValuationClass<I: RawIndex> {
    entries: To1<ValuationClassEntryIndex<I>, ValuationClassEntry>,
    name_to_entry: HashMap<String, usize>,
    next_free_index: usize,
    size_in_bits: usize,
}

impl<I: RawIndex> ValuationClass<I> {
    pub fn new() -> Self {
        Self {
            entries: To1::new(),
            name_to_entry: HashMap::new(),
            next_free_index: 0,
            size_in_bits: 0,
        }
    }

    pub fn add(&mut self, entry: ValuationEntryDescription) -> ValuationClassEntryIndex<I> {
        let size = entry.calculate_size();
        let entry = ValuationClassEntry {
            name: entry.name,
            variable_type: entry.variable_type,
            is_optional: entry.is_optional,
            value_offset: entry.value_offset,
            location: (self.next_free_index..self.next_free_index + size.nominal_size),
        };
        self.next_free_index += size.stored_size;
        self.size_in_bits = self.next_free_index;
        self.entries.add_unchecked(entry)
    }

    pub fn get(&self, index: ValuationClassEntryIndex<I>) -> &ValuationClassEntry {
        &self.entries[index]
    }

    pub fn size_in_bits(&self) -> usize {
        self.size_in_bits
    }
}

pub struct ValuationEntryDescription {
    name: String,
    variable_type: Type,
    is_optional: bool,
    value_offset: i64,
    size: Option<usize>,
}

impl ValuationEntryDescription {
    fn calculate_size(&self) -> VariableSize {
        let optional_bit = if self.is_optional { 1 } else { 0 };
        match self.size {
            None => {
                let default_size = match self.variable_type {
                    Type::Bool => 1,
                    Type::Int => 64,
                    Type::Uint => 64,
                    Type::Double => 64,
                    Type::Rational => 128,
                    Type::String => 64,
                };
                VariableSize {
                    nominal_size: default_size,
                    stored_size: default_size + optional_bit,
                }
            }
            Some(size) => {
                match self.variable_type {
                    // TODO: Allow valuations beyond the UMB standard. For example, usually it is
                    //  not necessary to use 64 bits for string indices.
                    Type::Bool => {
                        // Surprisingly, bools are allowed to use multiple bits for storage
                    }
                    Type::Int => {}
                    Type::Uint => {}
                    Type::Double => {
                        assert_eq!(size, 64, "Doubles must be stored in 64 bits");
                    }
                    Type::Rational => {
                        assert_eq!(
                            size % 2,
                            0,
                            "Rationals must use an even number of bits for storage"
                        );
                    }
                    Type::String => {
                        assert_eq!(size, 64, "Strings must use 64 bits for storage");
                    }
                }
                VariableSize {
                    nominal_size: size,
                    stored_size: size + optional_bit,
                }
            }
        }
    }
}

struct VariableSize {
    nominal_size: usize,
    stored_size: usize, // Includes a bit if the variable is optional
}

pub struct ValuationClassEntry {
    pub name: String,
    pub variable_type: Type,
    pub is_optional: bool,
    pub value_offset: i64,
    pub location: Range<usize>,
}

impl ValuationClassEntry {}

pub struct SizedType {
    variable_type: Type,
    size: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Type {
    Bool,
    Int,
    Uint,
    Double,
    Rational,
    String,
}
