use crate::expressions::{ValuationSource, VariableType};
use crate::variables::valuation_map::ValuationMapEntry;
use prism_model::VariableReference;
use probabilistic_models::ValuationClassEntryIndex;
use probabilistic_models::valuations::{ValuationBits, ValuationEntry};
use typed_index_collections::{Index, RawIndex};

pub struct ConstAndVarValuationSource<'a, 'b, 'c, 'd, I: RawIndex> {
    map: &'a super::ValuationMap<ValuationClassEntryIndex<I>>,
    const_valuation: &'b super::ConstValuations,
    details: &'c super::VariableDetails<I>,
    var_valuation: &'d ValuationEntry<'d, I>,
}

impl<'a, 'b, 'c, 'd, I: RawIndex> ConstAndVarValuationSource<'a, 'b, 'c, 'd, I> {
    pub fn new(
        map: &'a super::ValuationMap<ValuationClassEntryIndex<I>>,
        const_valuation: &'b super::ConstValuations,
        details: &'c super::VariableDetails<I>,
        var_valuation: &'d ValuationEntry<'d, I>,
    ) -> Self {
        Self {
            map,
            const_valuation,
            var_valuation,
            details,
        }
    }
}

impl<'a, 'b, 'c, 'd, I: RawIndex> ValuationSource
    for &ConstAndVarValuationSource<'a, 'b, 'c, 'd, I>
{
    fn get_int(&self, index: VariableReference) -> i64 {
        match self.map[index.index] {
            ValuationMapEntry::Const(i) => self.const_valuation[i].as_int(),
            ValuationMapEntry::Var(i) => self.var_valuation.evaluate_int(i),
        }
    }

    fn get_bool(&self, index: VariableReference) -> bool {
        match self.map[index.index] {
            ValuationMapEntry::Const(i) => self.const_valuation[i].as_bool(),
            ValuationMapEntry::Var(i) => self.var_valuation.evaluate_bool(i),
        }
    }

    fn get_float(&self, index: VariableReference) -> f64 {
        match self.map[index.index] {
            ValuationMapEntry::Const(i) => self.const_valuation[i].as_float(),
            ValuationMapEntry::Var(i) => self.var_valuation.evaluate_double(i),
        }
    }

    fn get_type(&self, index: VariableReference) -> VariableType {
        use crate::variables::const_valuations::ConstValuation;
        match self.map[index.index] {
            ValuationMapEntry::Const(i) => match self.const_valuation[i] {
                ConstValuation::Int(_) => VariableType::Int,
                ConstValuation::Bool(_) => VariableType::Bool,
                ConstValuation::Float(_) => VariableType::Float,
            },
            ValuationMapEntry::Var(i) => self.details[i].variable_type,
        }
    }
}

impl<'a, 'b, 'c, 'd, I: RawIndex> ValuationSource
    for ConstAndVarValuationSource<'a, 'b, 'c, 'd, I>
{
    fn get_int(&self, index: VariableReference) -> i64 {
        (&self).get_int(index)
    }

    fn get_bool(&self, index: VariableReference) -> bool {
        (&self).get_bool(index)
    }

    fn get_float(&self, index: VariableReference) -> f64 {
        (&self).get_float(index)
    }

    fn get_type(&self, index: VariableReference) -> VariableType {
        (&self).get_type(index)
    }
}
