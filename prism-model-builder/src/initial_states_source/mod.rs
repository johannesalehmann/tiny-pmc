mod entire_state_space;
pub use entire_state_space::StartFromEveryState;

mod reachable;
pub use reachable::StartFromInitialStates;

use crate::expression_context::ExpressionContext;
use crate::variables::ModelVariableInfo;
use prism_model::{Identifier, Model, Span, VariableRange, VariableReference};
use probabilistic_models::valuations::{BareStandaloneValuation, StandaloneValuation};
use typed_index_collections::Index;

#[derive(PartialEq, Debug)]
enum VariableValue {
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

    fn initial_values(&mut self) -> Vec<(Self::ClassEntryIdx, VariableValue)> {
        let mut values = Vec::new();

        let (model, info, eval) = self.info();
        for (i, variable) in model.variable_manager.variables.iter().enumerate() {
            if let Some(index) = &info.valuation_map.map_to_variable(i) {
                let value = match variable.range {
                    VariableRange::BoundedInt { .. } => {
                        VariableValue::Int(match &variable.initial_value {
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
                        VariableValue::Int(match &variable.initial_value {
                            None => panic!("Unbounded int must have init expression"),
                            Some(initial) => {
                                let value = eval
                                    .evaluate_int(initial, &info.get_const_only_valuation_source());
                                value
                            }
                        })
                    }
                    VariableRange::Boolean { .. } => {
                        VariableValue::Bool(match &variable.initial_value {
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
                        VariableValue::Float(match &variable.initial_value {
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

    fn min_values(&mut self) -> (Vec<(Self::ClassEntryIdx, VariableValue)>, bool) {
        let mut values = Vec::new();
        let mut exists = true;

        let (model, info, _) = self.info();
        for (i, variable) in model.variable_manager.variables.iter().enumerate() {
            if let Some(index) = &info.valuation_map.map_to_variable(i) {
                let value = match variable.range {
                    VariableRange::BoundedInt { .. } => {
                        VariableValue::Int(if let Some((min, max)) = info.details[*index].bounds {
                            if min > max {
                                exists = false;
                            }
                            min
                        } else {
                            panic!("Variable bounds list is inconsistent");
                        })
                    }
                    VariableRange::UnboundedInt { .. } => {
                        panic!(
                            "Cannot build entire model state space if the model contains unbounded variables"
                        );
                    }
                    VariableRange::Boolean { .. } => VariableValue::Bool(false),
                    VariableRange::Float { .. } => {
                        panic!(
                            "Cannot build entire model state space if the model contains float variables"
                        );
                    }
                };
                values.push((*index, value))
            }
        }
        (values, exists)
    }

    fn inc_values(&mut self, values: &mut Vec<(Self::ClassEntryIdx, VariableValue)>) -> bool {
        let (_, info, _) = self.info();

        for i in 0..values.len() {
            match &mut values[i] {
                (_, VariableValue::Bool(val)) => {
                    if *val == false {
                        *val = true;
                        return true;
                    } else {
                        *val = false;
                    }
                }
                (index, VariableValue::Int(val)) => {
                    if let Some((min, max)) = info.details[*index].bounds {
                        if *val < max {
                            *val += 1;
                            return true;
                        } else {
                            *val = min;
                        }
                    } else {
                        panic!("Variable bounds list is inconsistent");
                    }
                }
                (_, VariableValue::Float(_)) => {
                    unreachable!()
                }
            }
        }
        false
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
