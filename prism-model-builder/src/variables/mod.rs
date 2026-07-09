mod const_only_valuation_source;
pub use const_only_valuation_source::ConstOnlyValuationSource;

mod valuation_source;
pub use valuation_source::ConstAndVarValuationSource;

mod const_valuations;
pub use const_valuations::ConstValuation;

mod valuation_map;
mod variable_details;

use const_valuations::*;
use valuation_map::*;

use crate::variables::variable_details::VariableDetails;
use crate::{ExpressionContext, ModelBuildingError, UserProvidedConstValue};
use prism_model::{Identifier, Model, Span, VariableRange, VariableReference};
use probabilistic_models::valuations::{ValuationEntry, Valuations};
use probabilistic_models::{StateIndex, ValuationClassEntryIndex, ValuationClassIndex};
use std::collections::HashMap;
use typed_index_collections::{Index, RawIndex, To1};

pub struct ModelVariableInfo<I: RawIndex> {
    pub valuation_map: ValuationMap<ValuationClassEntryIndex<I>>,
    const_valuations: ConstValuations,
    pub details: VariableDetails<I>,
    pub class_index: ValuationClassIndex<I>,
}

impl<I: RawIndex> ModelVariableInfo<I> {
    /// Generates a valuation source for use in tests.
    ///
    /// The valuation source contains the following items.
    ///
    /// * `[0]`: Variable with index 0 of type `float` with name `float_var`
    /// * `[1]`: Variable with index 1 of type `int` with name `int_var` and bounds -10, 15
    /// * `[2]`: Const with index 0 of type `int` with value `-5`
    /// * `[3]`: Const with index 1 of type `bool` with value `true`
    /// * `[4]`: Variable with index 2 of type `bool` with name `bool_var`
    /// * `[5]`: Const with index 2 of type `float` with value `1.23`
    #[cfg(test)]
    pub fn with_mock_values() -> Self {
        // TODO: Do we need this information in any of the tests? Before restructuring, this info
        //  was stored in ModelVariableInfo and now it has nowhere to go
        // let mut builder = V::get_context_builder();
        // builder.register_float("float_var".to_string());
        // builder.register_bounded_int("int_var".to_string(), -10, 15);
        // builder.register_bool("bool_var".to_string());

        ModelVariableInfo {
            valuation_map: ValuationMap::with_mock_values(),
            const_valuations: ConstValuations::with_mock_values(),
            details: VariableDetails::with_mock_values(),
            class_index: ValuationClassIndex::from_raw(I::zero()),
        }
    }

    pub fn new<S: Span, E, EC: ExpressionContext<E>>(
        model: &Model<VariableReference, S, E, Identifier<S>>,
        user_provided_consts: &HashMap<String, UserProvidedConstValue>,
        expression_context: &mut EC,
        valuations: &mut Valuations<I, StateIndex<I>>,
    ) -> Result<Self, ModelBuildingError> {
        let variables = &model.variable_manager;

        let const_valuations =
            ConstValuations::new(variables, user_provided_consts, expression_context);
        let valuation_map = ValuationMap::new(variables);
        let details = VariableDetails::new(
            variables,
            &valuation_map,
            &const_valuations,
            expression_context,
        );
        let (valuation_map, class) =
            valuation_map.assign_variable_indices(&model.variable_manager, &details);
        let class_index = valuations.add_class(class);

        Ok(Self {
            valuation_map,
            const_valuations,
            details,
            class_index,
        })
    }

    pub fn get_const_only_valuation_source(
        &self,
    ) -> ConstOnlyValuationSource<'_, '_, ValuationClassEntryIndex<I>> {
        ConstOnlyValuationSource::new(&self.valuation_map, &self.const_valuations)
    }

    pub fn get_valuation_source<'a, 'b>(
        &'a self,
        valuation: &'b ValuationEntry<'b, I>,
    ) -> ConstAndVarValuationSource<'a, 'a, 'a, 'b, I> {
        ConstAndVarValuationSource::new(
            &self.valuation_map,
            &self.const_valuations,
            &self.details,
            valuation,
        )
    }

    pub fn value_of_const(&self, reference: VariableReference) -> Option<ConstValuation> {
        Some(self.const_valuations[self.valuation_map.map_to_constant(reference.index)?])
    }
}
