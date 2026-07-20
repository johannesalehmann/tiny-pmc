use super::{Context, InitialStateSource, StartFromInitialStates, VariableValue};
use crate::ModelBuilder;
use prism_model::Span;
use probabilistic_models::valuations::ValuationBitsMut;

#[derive(Default)]
pub struct StartFromEveryState {}

impl InitialStateSource for StartFromEveryState {
    fn mark_initial_states<'a, IniCreator: Context>(&self, state_creator: &mut IniCreator) {
        let initial_values = state_creator.initial_values();
        let (mut values, mut more_values) = state_creator.min_values();
        while more_values {
            let mut valuation = state_creator.create_valuation();
            let mut is_initial = true;
            for ((initial_index, initial_value), (index, value)) in
                initial_values.iter().zip(values.iter())
            {
                match value {
                    VariableValue::Int(val) => {
                        valuation.set_int(*index, *val);
                    }
                    VariableValue::Float(_) => {
                        unreachable!()
                    }
                    VariableValue::Bool(val) => {
                        valuation.set_bool(*index, *val);
                    }
                }
                assert_eq!(
                    *initial_index, *index,
                    "Initial values must be given in the same order as minimal values"
                );
                if *initial_value != *value {
                    is_initial = false;
                }
            }
            state_creator.add_state(valuation.bare(), is_initial);
            more_values = state_creator.inc_values(&mut values);
        }
    }
}

impl<
    'a,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    B: crate::bases::BaseModelBuilder,
    IB: crate::initial_states_builder::InitialStatesBuilder<StateIdx = B::StateIdx>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
> ModelBuilder<'a, S, Q, L, StartFromEveryState, B, IB, APs>
{
    pub fn with_reachable_state_space(
        self,
    ) -> ModelBuilder<'a, S, Q, L, StartFromInitialStates, B, IB, APs> {
        self.map_initial_state_source(StartFromInitialStates::default())
    }
}
