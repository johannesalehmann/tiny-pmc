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

    fn compatible_with_state_space_restriction(&self) -> bool {
        true
    }
}

impl<
    'a,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    B: crate::bases::BaseModelBuilder,
    IB: crate::initial_states_builder::InitialStatesBuilder<Ix = B::Ix>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<Ix = B::Ix>,
    CL: crate::choice_labels::ChoiceLabelBuilder<Ix = B::Ix>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = B::Ix>,
> ModelBuilder<'a, S, Q, L, StartFromInitialStates, B, IB, APs, CL, Rew>
{
    pub fn with_full_state_space(
        self,
    ) -> ModelBuilder<'a, S, Q, L, StartFromEveryState, B, IB, APs, CL, Rew> {
        self.map_initial_state_source(StartFromEveryState::default())
    }
}
