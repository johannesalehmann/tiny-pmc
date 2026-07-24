use super::{Context, InitialStateSource, StartFromEveryState, VariableValue};
use crate::ModelBuilder;
use prism_model::Span;
use probabilistic_models::valuations::ValuationBitsMut;

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
                VariableValue::Int(val) => valuation.set_int(index, val),
                VariableValue::Float(val) => valuation.set_double(index, val),
                VariableValue::Bool(val) => valuation.set_bool(index, val),
            }
        }
        state_creator.add_state(valuation.bare(), true);
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
    CL: crate::choice_labels::ChoiceLabelBuilder<ChoiceIdx = B::ChoiceIdx>,
> ModelBuilder<'a, S, Q, L, StartFromInitialStates, B, IB, APs, CL>
{
    pub fn with_full_state_space(
        self,
    ) -> ModelBuilder<'a, S, Q, L, StartFromEveryState, B, IB, APs, CL> {
        self.map_initial_state_source(StartFromEveryState::default())
    }
}
