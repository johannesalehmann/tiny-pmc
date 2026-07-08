use crate::index::RawIndex;
use crate::to1::To1;
use crate::valuations::class::{Type, ValuationClass, ValuationClassEntry};
use crate::{Index, ValuationClassEntryIndex, ValuationClassIndex, ValuationIndex};
use num_traits::ToBytes;
use std::marker::PhantomData;
use std::ops::Range;

mod bits;
use crate::valuations::bits::SetBits;
use bits::GetBits;

mod class;

pub struct Valuations<I: RawIndex, From: Index> {
    classes: To1<ValuationClassIndex<I>, ValuationClass<I>>,
    valuations: To1<ValuationClassIndex<I>, ValuationClassData<I>>,
    entity_to_class: To1<From, ValuationClassIndex<I>>,
    entity_to_index: To1<From, ValuationIndex<I>>,
}

impl<I: RawIndex, From: Index> Valuations<I, From> {
    pub fn add_class(&mut self, class: ValuationClass<I>) -> ValuationClassIndex<I> {
        let bits = class.size_in_bits();
        let index = self.classes.add(class);
        self.valuations.add(ValuationClassData::new(bits));
        index
    }

    pub fn entry(&self, entity: From) -> ValuationEntry<I> {
        let class_index = self.entity_to_class[entity];
        let index = self.entity_to_index[entity];
        ValuationEntry {
            class: &self.classes[class_index],
            class_data: &self.valuations[class_index],
            index,
        }
    }

    pub fn entry_mut(&mut self, entity: From) -> ValuationEntryMut<I> {
        let class_index = self.entity_to_class[entity];
        let index = self.entity_to_index[entity];
        ValuationEntryMut {
            class: &self.classes[class_index],
            class_data: &mut self.valuations[class_index],
            index,
        }
    }

    pub fn add_empty_valuation(&mut self, class: ValuationClassIndex<I>) -> ValuationIndex<I> {
        self.valuations[class].valuations.add_empty_entry()
    }

    pub fn create_standalone_valuation(
        &self,
        class_index: ValuationClassIndex<I>,
    ) -> StandaloneValuation<'_, I> {
        let class = &self.classes[class_index];
        let data = ValuationClassData::new(class.size_in_bits());
        StandaloneValuation {
            class_index,
            class,
            data,
        }
    }

    pub fn add_valuation(&mut self, valuation: StandaloneValuation<I>) -> ValuationIndex<I> {
        if !valuation.data.strings.is_empty() {
            panic!("Adding StandaloneValuations with strings is not yet supported");
        }
        self.valuations[valuation.class_index]
            .valuations
            .add_from_standalone(valuation)
    }
}

pub struct ValuationEntry<'a, I: RawIndex> {
    class: &'a ValuationClass<I>,
    class_data: &'a ValuationClassData<I>,
    index: ValuationIndex<I>,
}

pub struct ValuationEntryMut<'a, I: RawIndex> {
    class: &'a ValuationClass<I>,
    class_data: &'a mut ValuationClassData<I>,
    index: ValuationIndex<I>,
}

fn assert_optional(variable: &ValuationClassEntry, should_be_optional: bool) {
    match should_be_optional {
        true => {
            assert!(
                !variable.is_optional,
                "Cannot access optional variable with non-optional method"
            );
        }
        false => {
            assert!(
                variable.is_optional,
                "Cannot access non-optional variable with optional method"
            );
        }
    }
}

fn assert_type(variable: &ValuationClassEntry, expected_type: Type) {
    if variable.variable_type != expected_type {
        panic!(
            "Cannot evaluate variable of type {:?} as {:?}",
            variable.variable_type, expected_type
        );
    }
}

pub trait ValuationBits<'a, I: RawIndex + 'a> {
    fn class_and_index(
        &'a self,
    ) -> (
        &'a ValuationClass<I>,
        &'a ValuationClassData<I>,
        ValuationIndex<I>,
    );

    fn evaluate_bool(&'a self, variable_index: ValuationClassEntryIndex<I>) -> bool {
        let (class, class_data, index) = self.class_and_index();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        assert_type(variable, Type::Bool);
        class_data.valuations.bool(index, variable.location.clone())
    }

    fn evaluate_int(&'a self, variable_index: ValuationClassEntryIndex<I>) -> i64 {
        let (class, class_data, index) = self.class_and_index();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        if variable.variable_type == Type::Int {
            class_data.valuations.int(index, variable.location.clone()) + variable.value_offset
        } else if variable.variable_type == Type::Uint {
            class_data.valuations.uint(index, variable.location.clone()) as i64
                + variable.value_offset
        } else {
            panic!(
                "Cannot evaluate variable of type {:?} as int",
                variable.variable_type
            );
        }
    }

    fn evaluate_uint(&'a self, variable_index: ValuationClassEntryIndex<I>) -> u64 {
        let (class, class_data, index) = self.class_and_index();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        assert_type(variable, Type::Uint);
        assert!(
            variable.value_offset >= 0,
            "uint variables with negative offset have type int and cannot be evaluated as uint"
        );
        class_data.valuations.uint(index, variable.location.clone()) + variable.value_offset as u64
    }

    fn evaluate_double(&'a self, variable_index: ValuationClassEntryIndex<I>) -> f64 {
        let (class, class_data, index) = self.class_and_index();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        if variable.variable_type != Type::Double {
            panic!(
                "Cannot evaluate variable of type {:?} as double",
                variable.variable_type
            );
        }
        class_data
            .valuations
            .double(index, variable.location.clone())
    }

    fn evaluate_rational(&'a self, variable_index: ValuationClassEntryIndex<I>) -> (i64, u64) {
        let (class, class_data, index) = self.class_and_index();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        if variable.variable_type != Type::Rational {
            panic!(
                "Cannot evaluate variable of type {:?} as rational",
                variable.variable_type
            );
        }
        let num_address = variable.location.start..variable.location.len() / 2;
        let denom_address =
            variable.location.start + variable.location.len() / 2..variable.location.len();
        (
            class_data.valuations.int(index, num_address),
            class_data.valuations.uint(index, denom_address),
        )
    }

    fn evaluate_string(&'a self, variable_index: ValuationClassEntryIndex<I>) -> &'a str {
        let (class, class_data, index) = self.class_and_index();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        if variable.variable_type != Type::String {
            panic!(
                "Cannot evaluate variable of type {:?} as string",
                variable.variable_type
            );
        }
        let index = class_data.valuations.uint(index, variable.location.clone()) as usize;
        &class_data.strings[index]
    }
}

pub trait ValuationBitsMut<I: RawIndex> {
    fn class_and_index_mut(
        &mut self,
    ) -> (
        &ValuationClass<I>,
        &mut ValuationClassData<I>,
        ValuationIndex<I>,
    );

    fn set_bool(&mut self, variable_index: ValuationClassEntryIndex<I>, value: bool) {
        let (class, class_data, index) = self.class_and_index_mut();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        assert_type(variable, Type::Bool);
        class_data
            .valuations
            .set_bool(index, variable.location.clone(), value);
    }

    fn set_int(&mut self, variable_index: ValuationClassEntryIndex<I>, value: i64) {
        let (class, class_data, index) = self.class_and_index_mut();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        if variable.variable_type == Type::Int {
            class_data.valuations.set_int(
                index,
                variable.location.clone(),
                value - variable.value_offset,
            )
        } else if variable.variable_type == Type::Uint {
            if value < variable.value_offset {
                panic!(
                    "Can only store signed integer in unsigned field if the offset is sufficiently large. Value {value} should be greater or equal to offset {}",
                    variable.value_offset
                );
            }
            class_data.valuations.set_uint(
                index,
                variable.location.clone(),
                (value - variable.value_offset) as u64,
            )
        } else {
            panic!(
                "Cannot store variable of type {:?} in an int",
                variable.variable_type
            );
        }
    }

    fn set_uint(&mut self, variable_index: ValuationClassEntryIndex<I>, value: u64) {
        let (class, class_data, index) = self.class_and_index_mut();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        assert_type(variable, Type::Uint);
        class_data
            .valuations
            .set_uint(index, variable.location.clone(), value);
    }

    fn set_double(&mut self, variable_index: ValuationClassEntryIndex<I>, value: f64) {
        let (class, class_data, index) = self.class_and_index_mut();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        assert_type(variable, Type::Double);
        class_data
            .valuations
            .set_double(index, variable.location.clone(), value);
    }

    fn set_rational(
        &mut self,
        variable_index: ValuationClassEntryIndex<I>,
        (numerator, denominator): (i64, u64),
    ) {
        let (class, class_data, index) = self.class_and_index_mut();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        assert_type(variable, Type::Rational);
        let num_address = variable.location.start..variable.location.len() / 2;
        let denom_address =
            variable.location.start + variable.location.len() / 2..variable.location.len();
        class_data.valuations.set_int(index, num_address, numerator);
        class_data
            .valuations
            .set_uint(index, denom_address, denominator);
    }

    fn set_string(&mut self, variable_index: ValuationClassEntryIndex<I>, value: String) {
        // TODO: This might slowly fill up the string array with unused values. Perhaps reuse the
        //  values in some way or clear out old ones?
        let (class, class_data, index) = self.class_and_index_mut();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        assert_type(variable, Type::String);
        let string_index = class_data.strings.len() as u64;
        class_data.strings.push(value);
        class_data
            .valuations
            .set_uint(index, variable.location.clone(), string_index);
    }
}

impl<'a, I: RawIndex> ValuationBits<'a, I> for ValuationEntry<'a, I> {
    fn class_and_index(
        &self,
    ) -> (
        &'a ValuationClass<I>,
        &'a ValuationClassData<I>,
        ValuationIndex<I>,
    ) {
        (self.class, self.class_data, self.index)
    }
}
impl<'a, I: RawIndex> ValuationBitsMut<I> for ValuationEntryMut<'a, I> {
    fn class_and_index_mut(
        &mut self,
    ) -> (
        &ValuationClass<I>,
        &mut ValuationClassData<I>,
        ValuationIndex<I>,
    ) {
        (self.class, self.class_data, self.index)
    }
}

pub struct StandaloneValuation<'a, I: RawIndex> {
    class_index: ValuationClassIndex<I>,
    class: &'a ValuationClass<I>,
    data: ValuationClassData<I>,
}

impl<'a, I: RawIndex> ValuationBits<'a, I> for StandaloneValuation<'a, I> {
    fn class_and_index(
        &'a self,
    ) -> (
        &'a ValuationClass<I>,
        &'a ValuationClassData<I>,
        ValuationIndex<I>,
    ) {
        (self.class, &self.data, ValuationIndex::from_raw(I::zero()))
    }
}
impl<'a, I: RawIndex> ValuationBitsMut<I> for StandaloneValuation<'a, I> {
    fn class_and_index_mut(
        &mut self,
    ) -> (
        &ValuationClass<I>,
        &mut ValuationClassData<I>,
        ValuationIndex<I>,
    ) {
        (
            self.class,
            &mut self.data,
            ValuationIndex::from_raw(I::zero()),
        )
    }
}

pub struct ValuationClassData<I: RawIndex> {
    valuations: ValuationVector<I>,
    strings: Vec<String>,
}

impl<I: RawIndex> ValuationClassData<I> {
    pub fn new(data_size_in_bits: usize) -> Self {
        let valuations = if data_size_in_bits == 0 {
            ValuationVector::U0
        } else if data_size_in_bits <= 8 {
            ValuationVector::U8(To1::new())
        } else if data_size_in_bits <= 16 {
            ValuationVector::U16(To1::new())
        } else if data_size_in_bits <= 32 {
            ValuationVector::U32(To1::new())
        } else if data_size_in_bits <= 64 {
            ValuationVector::U64(To1::new())
        } else {
            let fields_per_valuation = data_size_in_bits / 64;
            ValuationVector::MultiField {
                fields: To1::new(),
                fields_per_valuation,
            }
        };

        Self {
            valuations,
            strings: Vec::new(),
        }
    }
}

pub enum ValuationVector<I: RawIndex> {
    U0, // For valuations without any data
    U8(To1<ValuationIndex<I>, u8>),
    U16(To1<ValuationIndex<I>, u16>),
    U32(To1<ValuationIndex<I>, u32>),
    U64(To1<ValuationIndex<I>, u64>),
    MultiField {
        fields: To1<ValuationIndex<I>, u64>,
        fields_per_valuation: usize,
    },
}

impl<I: RawIndex> ValuationVector<I> {
    pub fn add_empty_entry(&mut self) -> ValuationIndex<I> {
        match self {
            ValuationVector::U0 => ValuationIndex::from_raw(I::zero()),
            ValuationVector::U8(vals) => vals.add(0),
            ValuationVector::U16(vals) => vals.add(0),
            ValuationVector::U32(vals) => vals.add(0),
            ValuationVector::U64(vals) => vals.add(0),
            ValuationVector::MultiField {
                fields,
                fields_per_valuation,
            } => {
                let index = fields.add(0);
                for _ in 1..*fields_per_valuation {
                    fields.add(0);
                }
                index / I::from_usize(*fields_per_valuation)
            }
        }
    }

    pub fn add_from_standalone(&mut self, valuation: StandaloneValuation<I>) -> ValuationIndex<I> {
        match (self, valuation.data.valuations) {
            (ValuationVector::U0, ValuationVector::U0) => ValuationIndex::from_raw(I::zero()),
            (ValuationVector::U8(vals), ValuationVector::U8(new_vals)) => {
                vals.add(new_vals.take(ValuationIndex::from_raw(I::zero())).unwrap())
            }
            (ValuationVector::U16(vals), ValuationVector::U16(new_vals)) => {
                vals.add(new_vals.take(ValuationIndex::from_raw(I::zero())).unwrap())
            }
            (ValuationVector::U32(vals), ValuationVector::U32(new_vals)) => {
                vals.add(new_vals.take(ValuationIndex::from_raw(I::zero())).unwrap())
            }
            (ValuationVector::U64(vals), ValuationVector::U64(new_vals)) => {
                vals.add(new_vals.take(ValuationIndex::from_raw(I::zero())).unwrap())
            }
            (
                ValuationVector::MultiField {
                    fields,
                    fields_per_valuation,
                },
                ValuationVector::MultiField {
                    fields: new_fields,
                    fields_per_valuation: new_fields_per_valuation,
                },
            ) => {
                let mut new_fields = new_fields.into_iter();
                let index = fields.add(new_fields.next().unwrap());
                for new_field in new_fields {
                    fields.add(new_field);
                }
                index
            }
            _ => panic!(
                "New valuation does not have the same underlying type as the vector it is added to"
            ),
        }
    }

    fn int(&self, index: ValuationIndex<I>, range: Range<usize>) -> i64 {
        let bits = self.bits(index, range.clone());
        // Convert the two's complement (of given length) into the actual representation
        if bits & (1 << range.len()) != 0 {
            let mask = if range.len() == 64 {
                !0
            } else {
                (1 << (range.len())) - 1
            };
            // TODO: Doing the math in 128 bit space seems wasteful, but casting the u64 to i64
            //  before negation might cause an error (because the range of negative values in
            //  two's complement is one larger than the range of positive values).
            (-((!(bits - 1) & mask) as i128)) as i64
        } else {
            bits as i64
        }
    }

    fn set_int(&mut self, index: ValuationIndex<I>, range: Range<usize>, value: i64) {
        let bits = if value > 0 {
            value as u64
        } else {
            let mask: u64 = if range.len() == 64 {
                !0
            } else {
                (1 << range.len()) - 1
            };
            (!(-(value as i128)) & mask as i128) as u64 + 1
        };
        self.set_bits(index, range, bits)
    }

    fn uint(&self, index: ValuationIndex<I>, range: Range<usize>) -> u64 {
        self.bits(index, range)
    }
    fn set_uint(&mut self, index: ValuationIndex<I>, range: Range<usize>, value: u64) {
        self.set_bits(index, range, value)
    }

    fn double(&self, index: ValuationIndex<I>, range: Range<usize>) -> f64 {
        f64::from_le_bytes(self.bits(index, range).to_le_bytes())
    }
    fn set_double(&mut self, index: ValuationIndex<I>, range: Range<usize>, value: f64) {
        let bits = u64::from_le_bytes(value.to_le_bytes());
        self.set_bits(index, range, bits)
    }

    fn bool(&self, index: ValuationIndex<I>, range: Range<usize>) -> bool {
        let bits = self.bits(index, range);
        bits != 0
    }

    fn set_bool(&mut self, index: ValuationIndex<I>, range: Range<usize>, value: bool) {
        let bits = if value { 1 } else { 0 };
        self.set_bits(index, range, bits)
    }

    fn bits(&self, index: ValuationIndex<I>, range: Range<usize>) -> u64 {
        match self {
            ValuationVector::U0 => {
                panic!("Cannot access data in a valuation whose fields have length 0")
            }
            ValuationVector::U8(vals) => vals[index].bits(range),
            ValuationVector::U16(vals) => vals[index].bits(range),
            ValuationVector::U32(vals) => vals[index].bits(range),
            ValuationVector::U64(vals) => vals[index].bits(range),
            ValuationVector::MultiField {
                fields,
                fields_per_valuation,
            } => {
                let fields_per_valuation = I::from_usize(*fields_per_valuation);
                let start_index = index * fields_per_valuation;
                let end_index = start_index + fields_per_valuation;
                let fields = &fields[start_index..end_index];
                fields.bits(range)
            }
        }
    }

    fn bit(&self, index: ValuationIndex<I>, offset: usize) -> bool {
        match self {
            ValuationVector::U0 => {
                panic!("Cannot access data in a valuation whose fields have length 0")
            }
            ValuationVector::U8(vals) => vals[index].bit(offset),
            ValuationVector::U16(vals) => vals[index].bit(offset),
            ValuationVector::U32(vals) => vals[index].bit(offset),
            ValuationVector::U64(vals) => vals[index].bit(offset),
            ValuationVector::MultiField {
                fields,
                fields_per_valuation,
            } => {
                let fields_per_valuation = I::from_usize(*fields_per_valuation);
                let start_index = index * fields_per_valuation;
                let end_index = start_index + fields_per_valuation;
                let fields = &fields[start_index..end_index];
                fields.bit(offset)
            }
        }
    }

    fn set_bits(&mut self, index: ValuationIndex<I>, range: Range<usize>, bits: u64) {
        match self {
            ValuationVector::U0 => {
                panic!("Cannot modify data in a valuation whose fields have length 0")
            }
            ValuationVector::U8(vals) => vals[index].set_bits(range, bits),
            ValuationVector::U16(vals) => vals[index].set_bits(range, bits),
            ValuationVector::U32(vals) => vals[index].set_bits(range, bits),
            ValuationVector::U64(vals) => vals[index].set_bits(range, bits),
            ValuationVector::MultiField {
                fields,
                fields_per_valuation,
            } => {
                let fields_per_valuation = I::from_usize(*fields_per_valuation);
                let start_index = index * fields_per_valuation;
                let end_index = start_index + fields_per_valuation;
                let mut fields = &mut fields[start_index..end_index];
                fields.set_bits(range, bits)
            }
        }
    }

    fn set_bit(&mut self, index: ValuationIndex<I>, offset: usize, value: bool) {
        match self {
            ValuationVector::U0 => {
                panic!("Cannot modify data in a valuation whose fields have length 0")
            }
            ValuationVector::U8(vals) => vals[index].set_bit(offset, value),
            ValuationVector::U16(vals) => vals[index].set_bit(offset, value),
            ValuationVector::U32(vals) => vals[index].set_bit(offset, value),
            ValuationVector::U64(vals) => vals[index].set_bit(offset, value),
            ValuationVector::MultiField {
                fields,
                fields_per_valuation,
            } => {
                let fields_per_valuation = I::from_usize(*fields_per_valuation);
                let start_index = index * fields_per_valuation;
                let end_index = start_index + fields_per_valuation;
                let mut fields = &mut fields[start_index..end_index];
                fields.set_bit(offset, value)
            }
        }
    }
}
