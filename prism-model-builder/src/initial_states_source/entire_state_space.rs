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
    CL: crate::choice_labels::ChoiceLabelBuilder<ChoiceIdx = B::ChoiceIdx>,
> ModelBuilder<'a, S, Q, L, StartFromEveryState, B, IB, APs, CL>
{
    pub fn with_reachable_state_space(
        self,
    ) -> ModelBuilder<'a, S, Q, L, StartFromInitialStates, B, IB, APs, CL> {
        self.map_initial_state_source(StartFromInitialStates::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::ModelBuilder;
    use prism_model::{Expression, Identifier, Model, ModelType, VariableInfo, VariableRange};
    use probabilistic_models::StateIndex;
    use probabilistic_models::traits::{ReadStateSpace, ReadValuations};
    use probabilistic_models::valuations::ValuationBits;
    use typed_index_collections::Index;

    #[test]
    pub fn simple() {
        let mut prism: Model = Model::new(ModelType::mdp());
        let x_info = VariableInfo::global_var(Identifier::new("x").unwrap(), VariableRange::bool());
        prism.variable_manager.add_variable(x_info).unwrap();
        let y_info = VariableInfo::global_var(
            Identifier::new("y").unwrap(),
            VariableRange::bounded_int(Expression::int(-3), Expression::int(2)),
        );
        prism.variable_manager.add_variable(y_info).unwrap();
        let z_info = VariableInfo::global_var(Identifier::new("z").unwrap(), VariableRange::bool());
        prism.variable_manager.add_variable(z_info).unwrap();

        let model = ModelBuilder::new_mdp_builder(&mut prism)
            .with_full_state_space()
            .build();
        assert_eq!(model.states().len(), 2 * 6 * 2);
        let mut index = 0;
        for z in [false, true] {
            for y in [-3, -2, -1, 0, 1, 2] {
                for x in [false, true] {
                    let valuation = model.state_valuation(StateIndex::from_raw(index as u32));
                    assert_eq!(
                        valuation.evaluate_bool(valuation.class().index_by_name("x").unwrap()),
                        x
                    );
                    assert_eq!(
                        valuation.evaluate_int(valuation.class().index_by_name("y").unwrap()),
                        y
                    );
                    assert_eq!(
                        valuation.evaluate_bool(valuation.class().index_by_name("z").unwrap()),
                        z
                    );
                    index += 1;
                }
            }
        }
    }
}
