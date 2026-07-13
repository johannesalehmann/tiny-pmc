use crate::expressions::VariableType;
use crate::variables::variable_details::VariableDetails;
use prism_model::{Span, VariableManager};
use probabilistic_models::ValuationClassEntryIndex;
use probabilistic_models::valuations::{Type, ValuationClass, ValuationEntryDescription};
use typed_index_collections::{Index, RawIndex};

pub struct ValuationMap<ClassEntryIndex> {
    entries: Vec<ValuationMapEntry<ClassEntryIndex>>,
}

impl<ClassEntryIndex> ValuationMap<ClassEntryIndex> {
    #[cfg(test)]
    pub fn with_mock_values() -> Self
    where
        ClassEntryIndex: Index,
    {
        Self {
            entries: vec![
                ValuationMapEntry::Var(ClassEntryIndex::from_raw(
                    ClassEntryIndex::RawType::from_usize(0),
                )),
                ValuationMapEntry::Var(ClassEntryIndex::from_raw(
                    ClassEntryIndex::RawType::from_usize(1),
                )),
                ValuationMapEntry::Const(0),
                ValuationMapEntry::Const(1),
                ValuationMapEntry::Var(ClassEntryIndex::from_raw(
                    ClassEntryIndex::RawType::from_usize(2),
                )),
                ValuationMapEntry::Const(2),
            ],
        }
    }

    pub fn map_to_variable(&self, index: usize) -> Option<ClassEntryIndex>
    where
        ClassEntryIndex: Copy,
    {
        match &self.entries[index] {
            ValuationMapEntry::Const(_) => None,
            ValuationMapEntry::Var(i) => Some(*i),
        }
    }

    pub fn map_to_constant(&self, index: usize) -> Option<usize> {
        match self.entries[index] {
            ValuationMapEntry::Const(i) => Some(i),
            ValuationMapEntry::Var(_) => None,
        }
    }
}

impl ValuationMap<usize> {
    pub fn new<S: Span, E>(variables: &VariableManager<S, E>) -> Self {
        let mut entries = Vec::new();

        let mut consts_count = 0;
        let mut var_count = 0;
        for var in &variables.variables {
            if var.is_constant() {
                entries.push(ValuationMapEntry::Const(consts_count));
                consts_count += 1;
            } else {
                entries.push(ValuationMapEntry::Var(var_count));
                var_count += 1;
            }
        }
        Self { entries }
    }

    pub fn assign_variable_indices<ClassEntryIdx: Index, S: Span, E>(
        self,
        variables: &VariableManager<S, E>,
        details: &VariableDetails<ClassEntryIdx>,
    ) -> (ValuationMap<ClassEntryIdx>, ValuationClass<ClassEntryIdx>) {
        let mut entries = Vec::new();
        let mut class = ValuationClass::new();
        for (entry_index, entry) in self.entries.into_iter().enumerate() {
            match entry {
                ValuationMapEntry::Const(index) => entries.push(ValuationMapEntry::Const(index)),
                ValuationMapEntry::Var(index) => {
                    let variable = &variables.variables[entry_index];
                    let details = &details.details
                        [ClassEntryIdx::from_raw(ClassEntryIdx::RawType::from_usize(index))];
                    let variable_type = match details.variable_type {
                        VariableType::Int => {
                            if details.bounds.is_some() {
                                Type::Uint
                            } else {
                                Type::Int
                            }
                        }
                        VariableType::Bool => Type::Bool,
                        VariableType::Float => Type::Double,
                    };
                    let (value_offset, size) = if let (VariableType::Int, Some((min, max))) =
                        (details.variable_type, details.bounds)
                    {
                        let range: u64 = (max - min) as u64;
                        let bits = 64 - range.leading_zeros() as usize;
                        (min, Some(bits))
                    } else {
                        (0, None)
                    };

                    let index = class.add(ValuationEntryDescription {
                        name: variable.name.name.clone(),
                        variable_type,
                        is_optional: false,
                        value_offset,
                        size,
                    });
                    entries.push(ValuationMapEntry::Var(index))
                }
            }
        }
        (ValuationMap { entries }, class)
    }
}

impl<ClassEntryIndex> std::ops::Index<usize> for ValuationMap<ClassEntryIndex> {
    type Output = ValuationMapEntry<ClassEntryIndex>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

pub enum ValuationMapEntry<ClassEntryIndex> {
    Const(usize),
    Var(ClassEntryIndex),
}
