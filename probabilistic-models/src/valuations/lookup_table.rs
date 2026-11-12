use crate::valuations::{
    GetValuationClassIndex, GetValuationData, ValuationClassData, ValuationVector, Valuations,
};
use crate::{Ix, ValuationClassIndex, ValuationIndex};
use std::collections::HashMap;
use typed_index_collections::{Index, RawIndex, To1};

// TODO: Are two states allowed to have the same valuation? If yes, then from_valuations should
//  probably return Result<_, DuplicateValuations> instead of panicking. The model builder will
//  never produce such a model, but does UMB allow this configuration?

pub struct ValuationLookupTable<I: Ix, To> {
    classes: To1<ValuationClassIndex<I>, ValuationClassLookupTable<To>>,
}

pub enum ValuationClassLookupTable<To> {
    U0(Option<To>),
    U8(HashMap<u8, To>),
    U16(HashMap<u16, To>),
    U32(HashMap<u32, To>),
    U64(HashMap<u64, To>),
    MultiField(HashMap<Box<[u64]>, To>),
}

impl<I: Ix, To: Index> ValuationLookupTable<I, To> {
    pub fn from_valuations(valuations: &Valuations<I, To>) -> Self {
        let mut classes = To1::with_capacity(valuations.classes.len());
        for (class_index, class) in valuations.valuations.enumerate() {
            classes.add_checked(
                class_index,
                ValuationClassLookupTable::with_default_indices(class),
            );
        }
        for ((entity, &class_idx), &val_idx) in valuations
            .entity_to_class
            .enumerate()
            .zip(valuations.entity_to_index.iter())
        {
            classes[class_idx].add(
                val_idx,
                &valuations.valuations[class_idx].valuations,
                entity,
            )
        }
        Self { classes }
    }

    pub fn get<
        Val: GetValuationData<ValuationIndex<I>> + GetValuationClassIndex<ValuationClassIndex<I>>,
    >(
        &self,
        valuation: &Val,
    ) -> Option<To> {
        self.classes
            .get(valuation.valuation_class_index())?
            .get(valuation)
    }
}
impl<To: Index> ValuationClassLookupTable<To> {
    pub fn with_default_indices<ValuationIdx: Index>(
        valuations: &ValuationClassData<ValuationIdx>,
    ) -> Self {
        // This doesn't work because valuations store string indices. The string indices of a
        // StandaloneValuation don't match those in the model's valuation class. To fix this,
        // we'd have to manually hash the strings if the valuation contains any. I'm not sure
        // whether it's worth implementing this, as String valuations can only occur via UMB
        // import.
        assert!(
            valuations.strings.is_empty(),
            "cannot generate `ValuationClassLookup` for valuations that contain strings"
        );
        match &valuations.valuations {
            ValuationVector::U0 => Self::U0(None),
            ValuationVector::U8(_) => Self::U8(HashMap::new()),
            ValuationVector::U16(_) => Self::U16(HashMap::new()),
            ValuationVector::U32(_) => Self::U32(HashMap::new()),
            ValuationVector::U64(_) => Self::U64(HashMap::new()),
            ValuationVector::MultiField { .. } => Self::MultiField(HashMap::new()),
        }
    }

    pub fn add<ValuationIdx: Index>(
        &mut self,
        valuation_index: ValuationIdx,
        valuation_data: &ValuationVector<ValuationIdx>,
        to: To,
    ) {
        const DUPLICATE_ERROR: &str = "valuation contains two states with same valuation";
        match (self, valuation_data) {
            (Self::U0(target), ValuationVector::U0) => {
                if valuation_index.raw() != ValuationIdx::RawType::zero() {
                    panic!(
                        "Valuation index {:?} out of bounds for 0-bit valuations",
                        valuation_index.raw()
                    );
                }
                assert!(target.is_none(), "{DUPLICATE_ERROR}");
                *target = Some(to)
            }
            (Self::U8(map), ValuationVector::U8(vals)) => {
                assert_eq!(
                    map.insert(vals[valuation_index], to),
                    None,
                    "{DUPLICATE_ERROR}"
                );
            }
            (Self::U16(map), ValuationVector::U16(vals)) => {
                assert_eq!(
                    map.insert(vals[valuation_index], to),
                    None,
                    "{DUPLICATE_ERROR}"
                );
            }
            (Self::U32(map), ValuationVector::U32(vals)) => {
                assert_eq!(
                    map.insert(vals[valuation_index], to),
                    None,
                    "{DUPLICATE_ERROR}"
                );
            }
            (Self::U64(map), ValuationVector::U64(vals)) => {
                assert_eq!(
                    map.insert(vals[valuation_index], to),
                    None,
                    "{DUPLICATE_ERROR}"
                );
            }
            (
                Self::MultiField(map),
                ValuationVector::MultiField {
                    fields,
                    fields_per_valuation,
                },
            ) => {
                let fields = Box::from(
                    &fields[ValuationIdx::from_raw(ValuationIdx::RawType::from_usize(
                        valuation_index.raw().as_usize() * fields_per_valuation,
                    ))
                        ..ValuationIdx::from_raw(ValuationIdx::RawType::from_usize(
                            (valuation_index.raw().as_usize() + 1) * fields_per_valuation,
                        ))],
                );
                assert_eq!(map.insert(fields, to), None, "{DUPLICATE_ERROR}");
            }
            _ => panic!(
                "The valuation does not have the same size as the entries of the valuation-to-entity lookup table for this class."
            ),
        }
    }

    pub fn get<ValuationIdx: Index, Val: GetValuationData<ValuationIdx>>(
        &self,
        valuation: &Val,
    ) -> Option<To> {
        let zero = ValuationIdx::from_raw(ValuationIdx::RawType::zero());
        match (self, &valuation.valuation_class_data().valuations) {
            (ValuationClassLookupTable::U0(to), ValuationVector::U0) => to.clone(),
            (ValuationClassLookupTable::U8(map), ValuationVector::U8(vals)) => {
                map.get(&vals[zero]).cloned()
            }
            (ValuationClassLookupTable::U16(map), ValuationVector::U16(vals)) => {
                map.get(&vals[zero]).cloned()
            }
            (ValuationClassLookupTable::U32(map), ValuationVector::U32(vals)) => {
                map.get(&vals[zero]).cloned()
            }
            (ValuationClassLookupTable::U64(map), ValuationVector::U64(vals)) => {
                map.get(&vals[zero]).cloned()
            }
            (
                ValuationClassLookupTable::MultiField(map),
                ValuationVector::MultiField {
                    fields,
                    fields_per_valuation,
                },
            ) => {
                let fields = &fields[zero..ValuationIdx::from_raw(
                    ValuationIdx::RawType::from_usize(*fields_per_valuation),
                )];
                map.get(fields).cloned()
            }
            _ => panic!(
                "The valuation does not have the same size as the entries of the valuation-to-entity lookup table for this class."
            ),
        }
    }
}
