use crate::ValuationIndex;
use crate::valuations::{GetValuationData, StandaloneValuation, ValuationVector};
use num_traits::Zero;
use std::collections::HashMap;
use typed_index_collections::{Index, RawIndex};

pub enum ValuationToEntity<To: Index> {
    U0,
    U8(HashMap<u8, To>),
    U16(HashMap<u16, To>),
    U32(HashMap<u32, To>),
    U64(HashMap<u64, To>),
    MultiField(HashMap<Vec<u64>, To>),
}

impl<To: Index> ValuationToEntity<To> {
    pub fn new(class_bit_width: usize) -> Self {
        if class_bit_width == 0 {
            Self::U0
        } else if class_bit_width <= 8 {
            Self::U8(HashMap::new())
        } else if class_bit_width <= 16 {
            Self::U16(HashMap::new())
        } else if class_bit_width <= 32 {
            Self::U32(HashMap::new())
        } else if class_bit_width <= 64 {
            Self::U64(HashMap::new())
        } else {
            Self::MultiField(HashMap::new())
        }
    }

    pub fn add<'a, ValuationIdx: Index, Val: GetValuationData<ValuationIdx>>(
        &mut self,
        valuation: &Val,
        to: To,
    ) {
        let zero = ValuationIdx::from_raw(ValuationIdx::RawType::zero());
        match (self, &valuation.valuation_class_data().valuations) {
            (ValuationToEntity::U0, ValuationVector::U0) => {
                if to.raw() != To::RawType::zero() {
                    panic!(
                        "Cannot store non-zero entity indices in the valuation-to-entity maps if the valuation has size 0."
                    )
                }
            }
            (ValuationToEntity::U8(map), ValuationVector::U8(vals)) => {
                assert_eq!(map.insert(vals[zero], to), None);
            }
            (ValuationToEntity::U16(map), ValuationVector::U16(vals)) => {
                assert_eq!(map.insert(vals[zero], to), None);
            }
            (ValuationToEntity::U32(map), ValuationVector::U32(vals)) => {
                assert_eq!(map.insert(vals[zero], to), None);
            }
            (ValuationToEntity::U64(map), ValuationVector::U64(vals)) => {
                assert_eq!(map.insert(vals[zero], to), None);
            }
            (
                ValuationToEntity::MultiField(map),
                ValuationVector::MultiField {
                    fields,
                    fields_per_valuation,
                },
            ) => {
                let fields = Vec::from(
                    &fields[zero..ValuationIdx::from_raw(ValuationIdx::RawType::from_usize(
                        *fields_per_valuation,
                    ))],
                );
                assert_eq!(map.insert(fields, to), None);
            }
            _ => panic!(
                "The valuation does not have the same size has the entries of the valuation-to-entity lookup table for this class."
            ),
        }
    }

    pub fn get<'a, ValuationIdx: Index, Val: GetValuationData<ValuationIdx>>(
        &self,
        valuation: &Val,
    ) -> Option<To> {
        let zero = ValuationIdx::from_raw(ValuationIdx::RawType::zero());
        match (self, &valuation.valuation_class_data().valuations) {
            (ValuationToEntity::U0, ValuationVector::U0) => Some(To::from_raw(To::RawType::zero())),
            (ValuationToEntity::U8(map), ValuationVector::U8(vals)) => {
                map.get(&vals[zero]).cloned()
            }
            (ValuationToEntity::U16(map), ValuationVector::U16(vals)) => {
                map.get(&vals[zero]).cloned()
            }
            (ValuationToEntity::U32(map), ValuationVector::U32(vals)) => {
                map.get(&vals[zero]).cloned()
            }
            (ValuationToEntity::U64(map), ValuationVector::U64(vals)) => {
                map.get(&vals[zero]).cloned()
            }
            (
                ValuationToEntity::MultiField(map),
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
                "The valuation does not have the same size has the entries of the valuation-to-entity lookup table for this class."
            ),
        }
    }
}
