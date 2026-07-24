use std::fmt::{Display, Formatter};
use std::ops::Range;
use typed_index_collections::{Index, RawIndex, SemiboundedIndexRange, To1, ValuePerIndexSource};

mod bits;
use crate::valuations::bits::SetBits;
use bits::GetBits;

mod class;
pub use class::{Type, ValuationClass, ValuationClassEntry, ValuationEntryDescription};

#[derive(Default)]
pub struct Valuations<EntityIdx: Index, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>
{
    classes: To1<ClassIdx, ValuationClass<ClassEntryIdx>>,
    valuations: To1<ClassIdx, ValuationClassData<ValuationIdx>>,
    entity_to_class: To1<EntityIdx, ClassIdx>,
    entity_to_index: To1<EntityIdx, ValuationIdx>,
}

impl<EntityIdx: Index, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>
    Valuations<EntityIdx, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    pub fn add_class(&mut self, class: ValuationClass<ClassEntryIdx>) -> ClassIdx {
        let bits = class.size_in_bits();
        let index = self.classes.add(class);
        self.valuations.add(ValuationClassData::new(bits));
        index
    }

    pub fn class(&self, class_index: ClassIdx) -> &ValuationClass<ClassEntryIdx> {
        &self.classes[class_index]
    }

    pub fn classes(&self) -> SemiboundedIndexRange<ClassIdx> {
        self.classes.keys()
    }

    pub fn entry(
        &self,
        entity: EntityIdx,
    ) -> ValuationEntry<'_, ClassIdx, ClassEntryIdx, ValuationIdx> {
        let class_index = self.entity_to_class[entity];
        let index = self.entity_to_index[entity];
        ValuationEntry {
            class_index,
            class: &self.classes[class_index],
            class_data: &self.valuations[class_index],
            index,
        }
    }

    pub fn entry_mut(
        &mut self,
        entity: EntityIdx,
    ) -> ValuationEntryMut<'_, ClassIdx, ClassEntryIdx, ValuationIdx> {
        let class_index = self.entity_to_class[entity];
        let index = self.entity_to_index[entity];
        ValuationEntryMut {
            class_index,
            class: &self.classes[class_index],
            class_data: &mut self.valuations[class_index],
            index,
        }
    }

    pub fn add_empty_valuation(&mut self, entity: EntityIdx, class: ClassIdx) -> ValuationIdx {
        let index = self.valuations[class].valuations.add_empty_entry();
        self.entity_to_index.add_checked(entity, index);
        self.entity_to_class.add_checked(entity, class);
        index
    }

    pub fn create_standalone_valuation(
        &self,
        class_index: ClassIdx,
    ) -> StandaloneValuation<'_, ClassIdx, ClassEntryIdx, ValuationIdx> {
        let class = &self.classes[class_index];
        let data = ValuationClassData::new(class.size_in_bits());
        StandaloneValuation {
            class_index,
            class,
            data,
        }
    }

    pub fn add_valuation<Val: GetValuationClassIndex<ClassIdx> + GetValuationData<ValuationIdx>>(
        &mut self,
        entity: EntityIdx,
        valuation: &Val,
    ) -> ValuationIdx {
        if !valuation.valuation_class_data().strings.is_empty() {
            panic!("Adding StandaloneValuations with strings is not yet supported");
        }
        let class = valuation.valuation_class_index();
        let index = self.valuations[class]
            .valuations
            .add_from_standalone(valuation);
        self.entity_to_index.add_checked(entity, index);
        self.entity_to_class.add_checked(entity, class);
        index
    }
}

pub struct ValuationEntry<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index> {
    // TODO: The class index is only used when cloning the valuation into a standalone valuation.
    //  Is there some way of avoiding the need to always drag this around?
    class_index: ClassIdx,
    class: &'a ValuationClass<ClassEntryIdx>,
    class_data: &'a ValuationClassData<ValuationIdx>,
    index: ValuationIdx,
}

pub struct ValuationEntryMut<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index> {
    class_index: ClassIdx,
    class: &'a ValuationClass<ClassEntryIdx>,
    class_data: &'a mut ValuationClassData<ValuationIdx>,
    index: ValuationIdx,
}

impl<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>
    ValuationEntry<'a, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    pub fn clone_into_standalone_valuation(
        &self,
    ) -> StandaloneValuation<'_, ClassIdx, ClassEntryIdx, ValuationIdx> {
        StandaloneValuation::from_valuation_entry(self)
    }
}
impl<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>
    ValuationEntryMut<'a, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    pub fn clone_into_standalone_valuation(
        &self,
    ) -> StandaloneValuation<'_, ClassIdx, ClassEntryIdx, ValuationIdx> {
        StandaloneValuation::from_valuation_entry(&ValuationEntry {
            class_index: self.class_index,
            class: self.class,
            class_data: self.class_data,
            index: self.index,
        })
    }
}
impl<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index> Display
    for ValuationEntry<'a, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.write(f)
    }
}

fn assert_optional(variable: &ValuationClassEntry, should_be_optional: bool) {
    match should_be_optional {
        true => {
            assert!(
                variable.is_optional,
                "Cannot access optional variable with non-optional method"
            );
        }
        false => {
            assert!(
                !variable.is_optional,
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

pub trait ValuationBits<ClassEntryIdx: Index, ValuationIdx: Index> {
    fn class_and_index(
        &self,
    ) -> (
        &ValuationClass<ClassEntryIdx>,
        &ValuationClassData<ValuationIdx>,
        ValuationIdx,
    );

    fn class(&self) -> &ValuationClass<ClassEntryIdx>;

    fn variables(&self) -> &To1<ClassEntryIdx, ValuationClassEntry> {
        self.class().entries()
    }

    fn write(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut first = true;
        for (index, variable) in self.variables().enumerate() {
            if !first {
                write!(f, ",")?;
            }
            first = false;
            write!(f, "{}=", variable.name)?;
            match variable.variable_type {
                Type::Bool => {
                    write!(f, "{}", self.evaluate_bool(index))?;
                }
                Type::Int => {
                    write!(f, "{}", self.evaluate_int(index))?;
                }
                Type::Uint => {
                    // We could instead always use evaluate_int, but this would fail in the
                    // (probably rare) cases where the value can be contained in a u64, but not
                    // an i64
                    if variable.value_offset < 0 {
                        write!(f, "{}", self.evaluate_int(index))?;
                    } else {
                        write!(f, "{}", self.evaluate_uint(index))?;
                    }
                }
                Type::Double => {
                    write!(f, "{}", self.evaluate_double(index))?;
                }
                Type::Rational => {
                    let (num, denom) = self.evaluate_rational(index);
                    write!(f, "{num}/{denom}")?;
                }
                Type::String => {
                    write!(f, "{}", self.evaluate_string(index))?;
                }
            }
        }
        Ok(())
    }

    fn evaluate_bool(&self, variable_index: ClassEntryIdx) -> bool {
        let (class, class_data, index) = self.class_and_index();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        assert_type(variable, Type::Bool);
        class_data.valuations.bool(index, variable.location.clone())
    }

    fn evaluate_int(&self, variable_index: ClassEntryIdx) -> i64 {
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

    fn evaluate_uint(&self, variable_index: ClassEntryIdx) -> u64 {
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

    fn evaluate_double(&self, variable_index: ClassEntryIdx) -> f64 {
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

    fn evaluate_rational(&self, variable_index: ClassEntryIdx) -> (i64, u64) {
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

    fn evaluate_string<'a>(&'a self, variable_index: ClassEntryIdx) -> &'a str
    where
        ClassEntryIdx: 'a,
        ValuationIdx: 'a,
    {
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

    fn equals_to<VB: ValuationBits<ClassEntryIdx, ValuationIdx>>(&self, other: &VB) -> bool {
        let own_class = self.class();
        let other_class = self.class();
        if own_class.entries().len() != other_class.entries().len() {
            return false;
        }
        for (index, variable) in own_class.entries().enumerate() {
            let equals = match variable.variable_type {
                Type::Bool => self.evaluate_bool(index) == other.evaluate_bool(index),
                Type::Int => self.evaluate_int(index) == other.evaluate_int(index),
                Type::Uint => {
                    // Evaluate as integers because of offset
                    self.evaluate_int(index) == other.evaluate_int(index)
                }
                Type::Double => self.evaluate_double(index) == other.evaluate_double(index),
                Type::Rational => self.evaluate_rational(index) == other.evaluate_rational(index),
                Type::String => self.evaluate_string(index) == other.evaluate_string(index),
            };
            if !equals {
                return false;
            }
        }
        true
    }
}

pub trait ValuationBitsMut<ClassEntryIdx: Index, ValuationIdx: Index> {
    fn class_and_index_mut(
        &mut self,
    ) -> (
        &ValuationClass<ClassEntryIdx>,
        &mut ValuationClassData<ValuationIdx>,
        ValuationIdx,
    );

    fn set_bool(&mut self, variable_index: ClassEntryIdx, value: bool) {
        let (class, class_data, index) = self.class_and_index_mut();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        assert_type(variable, Type::Bool);
        class_data
            .valuations
            .set_bool(index, variable.location.clone(), value);
    }

    fn set_int(&mut self, variable_index: ClassEntryIdx, value: i64) {
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

    fn set_uint(&mut self, variable_index: ClassEntryIdx, value: u64) {
        let (class, class_data, index) = self.class_and_index_mut();
        let variable = class.get(variable_index);
        assert_optional(variable, false);
        assert_type(variable, Type::Uint);
        class_data
            .valuations
            .set_uint(index, variable.location.clone(), value);
    }

    fn set_double(&mut self, variable_index: ClassEntryIdx, value: f64) {
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
        variable_index: ClassEntryIdx,
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

    fn set_string(&mut self, variable_index: ClassEntryIdx, value: String) {
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

impl<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>
    ValuationBits<ClassEntryIdx, ValuationIdx>
    for ValuationEntry<'a, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    fn class_and_index(
        &self,
    ) -> (
        &ValuationClass<ClassEntryIdx>,
        &ValuationClassData<ValuationIdx>,
        ValuationIdx,
    ) {
        (self.class, self.class_data, self.index)
    }

    fn class(&self) -> &ValuationClass<ClassEntryIdx> {
        self.class
    }
}
impl<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>
    ValuationBitsMut<ClassEntryIdx, ValuationIdx>
    for ValuationEntryMut<'a, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    fn class_and_index_mut(
        &mut self,
    ) -> (
        &ValuationClass<ClassEntryIdx>,
        &mut ValuationClassData<ValuationIdx>,
        ValuationIdx,
    ) {
        (self.class, self.class_data, self.index)
    }
}

pub trait GetValuationClassIndex<ClassIdx: Index> {
    fn valuation_class_index(&self) -> ClassIdx;
}
pub trait GetValuationData<ValuationIdx: Index> {
    fn valuation_class_data(&self) -> &ValuationClassData<ValuationIdx>;
}

pub struct StandaloneValuation<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index> {
    pub class_index: ClassIdx,
    pub class: &'a ValuationClass<ClassEntryIdx>,
    // TODO: Using ValuationClassData is both unclean (we only use the element at 0) and produces
    //  unnecessary allocations. It would be better to have a special version for this that is
    //  similar to ValuationClassData, but only contains a single entry instead of a vector. For
    //  all cases except for MultiFields, this would avoid allocating (and this structure is used
    //  in the inner model building loop, so it would be good to avoid allocations).
    pub data: ValuationClassData<ValuationIdx>,
}

impl<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>
    StandaloneValuation<'a, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    pub fn new(class_index: ClassIdx, class: &'a ValuationClass<ClassEntryIdx>) -> Self {
        let mut data = ValuationClassData::new(class.size_in_bits());
        data.valuations.add_empty_entry();
        Self {
            class_index,
            class,
            data,
        }
    }
    pub fn from_valuation_entry(
        entry: &ValuationEntry<'a, ClassIdx, ClassEntryIdx, ValuationIdx>,
    ) -> Self {
        if entry.class_data.strings.len() > 0 {
            panic!("`from_valuation_entry` cannot handle valuations that contain strings yet");
        }
        Self {
            class_index: entry.class_index,
            class: entry.class,
            data: ValuationClassData {
                valuations: entry
                    .class_data
                    .valuations
                    .copy_into_new_vector(entry.index),
                strings: Vec::new(), // TODO: Is there some way to avoid this allocation?
            },
        }
    }

    pub fn bare(self) -> BareStandaloneValuation<ClassIdx, ValuationIdx> {
        self.into()
    }
}
impl<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index> Display
    for StandaloneValuation<'a, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.write(f)
    }
}

impl<ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index> GetValuationClassIndex<ClassIdx>
    for StandaloneValuation<'_, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    fn valuation_class_index(&self) -> ClassIdx {
        self.class_index
    }
}

impl<ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index> GetValuationData<ValuationIdx>
    for StandaloneValuation<'_, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    fn valuation_class_data(&self) -> &ValuationClassData<ValuationIdx> {
        &self.data
    }
}

impl<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>
    ValuationBits<ClassEntryIdx, ValuationIdx>
    for StandaloneValuation<'a, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    fn class_and_index(
        &self,
    ) -> (
        &ValuationClass<ClassEntryIdx>,
        &ValuationClassData<ValuationIdx>,
        ValuationIdx,
    ) {
        (
            self.class,
            &self.data,
            ValuationIdx::from_raw(ValuationIdx::RawType::zero()),
        )
    }

    fn class(&self) -> &ValuationClass<ClassEntryIdx> {
        self.class
    }
}
impl<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>
    ValuationBitsMut<ClassEntryIdx, ValuationIdx>
    for StandaloneValuation<'a, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    fn class_and_index_mut(
        &mut self,
    ) -> (
        &ValuationClass<ClassEntryIdx>,
        &mut ValuationClassData<ValuationIdx>,
        ValuationIdx,
    ) {
        (
            self.class,
            &mut self.data,
            ValuationIdx::from_raw(ValuationIdx::RawType::zero()),
        )
    }
}

// A `BareStandaloneValuation` does not contain a reference to its class. This makes it much less
// capable, as reading and writing information requires the class, but avoids storing a reference,
// which may cause borrow-checker issues.
pub struct BareStandaloneValuation<ClassIdx: Index, ValuationIdx: Index> {
    pub class_index: ClassIdx,
    pub data: ValuationClassData<ValuationIdx>,
}

impl<'a, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>
    From<StandaloneValuation<'a, ClassIdx, ClassEntryIdx, ValuationIdx>>
    for BareStandaloneValuation<ClassIdx, ValuationIdx>
{
    fn from(value: StandaloneValuation<'a, ClassIdx, ClassEntryIdx, ValuationIdx>) -> Self {
        Self {
            class_index: value.class_index,
            data: value.data,
        }
    }
}

impl<ClassIdx: Index, ValuationIdx: Index> GetValuationClassIndex<ClassIdx>
    for BareStandaloneValuation<ClassIdx, ValuationIdx>
{
    fn valuation_class_index(&self) -> ClassIdx {
        self.class_index
    }
}

impl<ClassIdx: Index, ValuationIdx: Index> GetValuationData<ValuationIdx>
    for BareStandaloneValuation<ClassIdx, ValuationIdx>
{
    fn valuation_class_data(&self) -> &ValuationClassData<ValuationIdx> {
        &self.data
    }
}

pub struct ValuationClassData<ValuationIdx: Index> {
    pub valuations: ValuationVector<ValuationIdx>,
    pub strings: Vec<String>,
}

impl<ValuationIdx: Index> ValuationClassData<ValuationIdx> {
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

pub enum ValuationVector<ValuationIdx: Index> {
    U0, // For valuations without any data
    U8(To1<ValuationIdx, u8>),
    U16(To1<ValuationIdx, u16>),
    U32(To1<ValuationIdx, u32>),
    U64(To1<ValuationIdx, u64>),
    MultiField {
        fields: To1<ValuationIdx, u64>,
        fields_per_valuation: usize,
    },
}

impl<ValuationIdx: Index> ValuationVector<ValuationIdx> {
    pub fn add_empty_entry(&mut self) -> ValuationIdx {
        match self {
            ValuationVector::U0 => ValuationIdx::from_raw(ValuationIdx::RawType::zero()),
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
                index / ValuationIdx::RawType::from_usize(*fields_per_valuation)
            }
        }
    }

    pub fn add_from_standalone<Val: GetValuationData<ValuationIdx>>(
        &mut self,
        valuation: &Val,
    ) -> ValuationIdx {
        let zero = ValuationIdx::from_raw(ValuationIdx::RawType::zero());
        match (self, &valuation.valuation_class_data().valuations) {
            (ValuationVector::U0, ValuationVector::U0) => zero,
            (ValuationVector::U8(vals), ValuationVector::U8(new_vals)) => {
                vals.add(*new_vals.get(zero).unwrap())
            }
            (ValuationVector::U16(vals), ValuationVector::U16(new_vals)) => {
                vals.add(*new_vals.get(zero).unwrap())
            }
            (ValuationVector::U32(vals), ValuationVector::U32(new_vals)) => {
                vals.add(*new_vals.get(zero).unwrap())
            }
            (ValuationVector::U64(vals), ValuationVector::U64(new_vals)) => {
                vals.add(*new_vals.get(zero).unwrap())
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
                assert_eq!(*fields_per_valuation, *new_fields_per_valuation);
                let mut new_fields = new_fields.into_iter();
                let index = fields.add(*new_fields.next().unwrap());
                for &new_field in new_fields {
                    fields.add(new_field);
                }
                index
            }
            _ => panic!(
                "New valuation does not have the same underlying type as the vector it is added to"
            ),
        }
    }

    pub fn copy_into_new_vector(&self, index: ValuationIdx) -> Self {
        match self {
            ValuationVector::U0 => ValuationVector::U0,
            ValuationVector::U8(vals) => {
                ValuationVector::U8(To1::with_entries(vec![vals.get(index).unwrap().clone()]))
            }
            ValuationVector::U16(vals) => {
                ValuationVector::U16(To1::with_entries(vec![vals.get(index).unwrap().clone()]))
            }
            ValuationVector::U32(vals) => {
                ValuationVector::U32(To1::with_entries(vec![vals.get(index).unwrap().clone()]))
            }
            ValuationVector::U64(vals) => {
                ValuationVector::U64(To1::with_entries(vec![vals.get(index).unwrap().clone()]))
            }
            ValuationVector::MultiField {
                fields,
                fields_per_valuation,
            } => {
                let field_count = ValuationIdx::RawType::from_usize(*fields_per_valuation);
                let fields = &fields
                    [index * field_count..(index + ValuationIdx::RawType::one()) * field_count];
                ValuationVector::MultiField {
                    fields: To1::with_entries(fields.into_iter().cloned().collect::<Vec<_>>()),
                    fields_per_valuation: *fields_per_valuation,
                }
            }
        }
    }

    fn int(&self, index: ValuationIdx, range: Range<usize>) -> i64 {
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

    fn set_int(&mut self, index: ValuationIdx, range: Range<usize>, value: i64) {
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

    fn uint(&self, index: ValuationIdx, range: Range<usize>) -> u64 {
        self.bits(index, range)
    }
    fn set_uint(&mut self, index: ValuationIdx, range: Range<usize>, value: u64) {
        self.set_bits(index, range, value)
    }

    fn double(&self, index: ValuationIdx, range: Range<usize>) -> f64 {
        f64::from_le_bytes(self.bits(index, range).to_le_bytes())
    }
    fn set_double(&mut self, index: ValuationIdx, range: Range<usize>, value: f64) {
        let bits = u64::from_le_bytes(value.to_le_bytes());
        self.set_bits(index, range, bits)
    }

    fn bool(&self, index: ValuationIdx, range: Range<usize>) -> bool {
        let bits = self.bits(index, range);
        bits != 0
    }

    fn set_bool(&mut self, index: ValuationIdx, range: Range<usize>, value: bool) {
        let bits = if value { 1 } else { 0 };
        self.set_bits(index, range, bits)
    }

    fn bits(&self, index: ValuationIdx, range: Range<usize>) -> u64 {
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
                let fields_per_valuation = ValuationIdx::RawType::from_usize(*fields_per_valuation);
                let start_index = index * fields_per_valuation;
                let end_index = start_index + fields_per_valuation;
                let fields = &fields[start_index..end_index];
                fields.bits(range)
            }
        }
    }

    #[allow(unused)] // TODO: Remove the allow(unused) once optional values are supported.
    fn bit(&self, index: ValuationIdx, offset: usize) -> bool {
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
                let fields_per_valuation = ValuationIdx::RawType::from_usize(*fields_per_valuation);
                let start_index = index * fields_per_valuation;
                let end_index = start_index + fields_per_valuation;
                let fields = &fields[start_index..end_index];
                fields.bit(offset)
            }
        }
    }

    fn set_bits(&mut self, index: ValuationIdx, range: Range<usize>, bits: u64) {
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
                let fields_per_valuation = ValuationIdx::RawType::from_usize(*fields_per_valuation);
                let start_index = index * fields_per_valuation;
                let end_index = start_index + fields_per_valuation;
                let mut fields = &mut fields[start_index..end_index];
                fields.set_bits(range, bits)
            }
        }
    }

    #[allow(unused)]
    fn set_bit(&mut self, index: ValuationIdx, offset: usize, value: bool) {
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
                let fields_per_valuation = ValuationIdx::RawType::from_usize(*fields_per_valuation);
                let start_index = index * fields_per_valuation;
                let end_index = start_index + fields_per_valuation;
                let mut fields = &mut fields[start_index..end_index];
                fields.set_bit(offset, value)
            }
        }
    }
}
