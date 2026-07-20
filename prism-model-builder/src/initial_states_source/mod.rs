use crate::expression_context;
use crate::expression_context::ExpressionContext;
use crate::expressions::ValuationSource;
use crate::initial_states_builder::InitialStatesBuilder;
use crate::variables::ModelVariableInfo;
use prism_model::{Identifier, Model, Span, VariableInfo, VariableRange, VariableReference};
use probabilistic_models::valuations::{
    BareStandaloneValuation, StandaloneValuation, ValuationBitsMut, ValuationClass,
};
use typed_index_collections::Index;

enum InitialValue {
    Int(i64),
    Float(f64),
    Bool(bool),
}

pub trait Context {
    type Span: Span;
    type Expression;
    type ExpressionContext: ExpressionContext<Self::Expression>;
    type ClassIdx: Index;
    type ClassEntryIdx: Index;
    type ValuationIdx: Index;
    fn info(
        &mut self,
    ) -> (
        &Model<VariableReference, Self::Span, Self::Expression, Identifier<Self::Span>>,
        &ModelVariableInfo<Self::ClassIdx, Self::ClassEntryIdx>,
        &mut Self::ExpressionContext,
    );

    fn initial_values(&mut self) -> Vec<(Self::ClassEntryIdx, InitialValue)> {
        let mut values = Vec::new();

        let (model, info, eval) = self.info();
        for (i, variable) in model.variable_manager.variables.iter().enumerate() {
            if let Some(index) = &info.valuation_map.map_to_variable(i) {
                let value = match variable.range {
                    VariableRange::BoundedInt { .. } => {
                        InitialValue::Int(match &variable.initial_value {
                            None => {
                                if let Some((min, _)) = info.details[*index].bounds {
                                    min
                                } else {
                                    panic!("Variable bounds list is inconsistent");
                                }
                            }
                            Some(initial) => {
                                let value = eval
                                    .evaluate_int(initial, &info.get_const_only_valuation_source());
                                value
                            }
                        })
                    }
                    VariableRange::UnboundedInt { .. } => {
                        InitialValue::Int(match &variable.initial_value {
                            None => panic!("Unbounded int must have init expression"),
                            Some(initial) => {
                                let value = eval
                                    .evaluate_int(initial, &info.get_const_only_valuation_source());
                                value
                            }
                        })
                    }
                    VariableRange::Boolean { .. } => {
                        InitialValue::Bool(match &variable.initial_value {
                            None => false,
                            Some(initial) => {
                                let value = eval.evaluate_bool(
                                    initial,
                                    &info.get_const_only_valuation_source(),
                                );
                                value
                            }
                        })
                    }
                    VariableRange::Float { .. } => {
                        InitialValue::Float(match &variable.initial_value {
                            None => {
                                panic!(
                                    "Floats must have init expressions (I'm not sure whether this is PRISM-spec-compliant)"
                                )
                            }
                            Some(initial) => {
                                let value = eval.evaluate_float(
                                    initial,
                                    &info.get_const_only_valuation_source(),
                                );
                                value
                            }
                        })
                    }
                };
                values.push((*index, value))
            }
        }
        values
    }
    fn create_valuation(
        &self,
    ) -> StandaloneValuation<'_, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx>;
    fn add_state(
        &mut self,
        valuation: BareStandaloneValuation<Self::ClassIdx, Self::ValuationIdx>,
        is_initial: bool,
    );
}

pub trait InitialStateSource {
    fn mark_initial_states<'a, IniCreator: Context>(&self, state_creator: &mut IniCreator);
}

#[derive(Default)]
pub struct StartFromInitialStates {}

impl InitialStateSource for StartFromInitialStates {
    fn mark_initial_states<'a, IniCreator: Context>(&self, state_creator: &mut IniCreator) {
        let (model, _, _) = state_creator.info();
        if model.init_constraint.is_some() {
            panic!("Init constraints are not yet supported by the model builder");
        }

        let initial_values = state_creator.initial_values();

        let mut valuation = state_creator.create_valuation();

        for (index, value) in initial_values {
            match value {
                InitialValue::Int(val) => valuation.set_int(index, val),
                InitialValue::Float(val) => valuation.set_double(index, val),
                InitialValue::Bool(val) => valuation.set_bool(index, val),
            }
        }
        state_creator.add_state(valuation.bare(), true);
    }
}

#[derive(Default)]
pub struct StartFromEveryState {}
