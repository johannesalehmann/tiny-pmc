use crate::bases::BaseModelBuilder;
use crate::initial_states_builder::InitialStatesBuilder;
use crate::initial_states_source::InitialStateSource;
use crate::labels::LabelSource;
use crate::queries::QueryCollection;
use crate::rewards_builder::RewardsBuilder;
use crate::{ModelBuilder, atomic_propositions_builder, choice_labels};
use prism_model::{Expression, Span, VariableReference};

impl<
    'a,
    S: Span,
    Q: QueryCollection,
    L: LabelSource,
    IS: InitialStateSource,
    B: BaseModelBuilder,
    IB: InitialStatesBuilder<StateIdx = B::StateIdx>,
    APs: atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
    CL: choice_labels::ChoiceLabelBuilder<ChoiceIdx = B::ChoiceIdx>,
    Rew: RewardsBuilder<StateIdx = B::StateIdx, ChoiceIdx = B::ChoiceIdx>,
> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, CL, Rew>
{
    pub fn with_restricted_state_space(
        self,
        condition: Expression<VariableReference, S>,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, CL, Rew> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
            choice_labels: self.choice_labels,
            rewards: self.rewards,
            state_space_restriction: Some(condition),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ModelBuilder;
    use prism_model::Expression;
    use probabilistic_models::StateIndex;
    use probabilistic_models::traits::{ReadStateSpace, ReadValuations};
    use typed_index_collections::Index;

    const SIMPLE_MODEL: &'static str = r#"
        mdp
        module main
            x: [0..4] init 0;
            [] (x<4) -> (x'=x+1);
        endmodule"#;

    #[test]
    fn simple_restriction() {
        let source = SIMPLE_MODEL;
        let mut prism = prism_parser::parse_model(source).unwrap();
        let x = prism.variable_manager.get_reference_by_str("x").unwrap();
        let restriction = Expression::var_or_const(x).less_than(Expression::int(2));
        let model = ModelBuilder::new_mdp_builder(&mut prism)
            .with_restricted_state_space(restriction)
            .build();
        assert_eq!(model.states().len(), 3);
        for i in 0..2 {
            assert_eq!(
                model.state_valuation(StateIndex::from_raw(i)).to_string(),
                format!("x={i}")
            );
        }
    }

    #[test]
    fn no_restriction() {
        let source = SIMPLE_MODEL;
        let mut prism = prism_parser::parse_model(source).unwrap();
        let model = ModelBuilder::new_mdp_builder(&mut prism).build();
        assert_eq!(model.states().len(), 5);
        for i in 0..=4 {
            assert_eq!(
                model.state_valuation(StateIndex::from_raw(i)).to_string(),
                format!("x={i}")
            );
        }
    }

    #[test]
    fn true_restriction() {
        let source = SIMPLE_MODEL;
        let mut prism = prism_parser::parse_model(source).unwrap();
        let x = prism.variable_manager.get_reference_by_str("x").unwrap();
        let restriction = Expression::bool(true);
        let model = ModelBuilder::new_mdp_builder(&mut prism)
            .with_restricted_state_space(restriction)
            .build();
        assert_eq!(model.states().len(), 5);
        for i in 0..=4 {
            assert_eq!(
                model.state_valuation(StateIndex::from_raw(i)).to_string(),
                format!("x={i}")
            );
        }
    }

    #[test]
    fn unsatisfied_restriction() {
        let source = SIMPLE_MODEL;
        let mut prism = prism_parser::parse_model(source).unwrap();
        let x = prism.variable_manager.get_reference_by_str("x").unwrap();
        let restriction = Expression::var_or_const(x).greater_or_equal(Expression::int(2));
        let model = ModelBuilder::new_mdp_builder(&mut prism)
            .with_restricted_state_space(restriction)
            .build();
        assert_eq!(model.states().len(), 1);
        assert_eq!(
            model.state_valuation(StateIndex::from_raw(0)).to_string(),
            "x=0"
        );
    }

    const SYNC_MODEL: &'static str = r#"
        mdp
        module a
            x: [0..4] init 0;
            [sync] (x<4) -> (x'=x+1);
        endmodule
        module b
            y: [0..4] init 0;
            [sync] (y<4) -> (y'=y+1);
        endmodule"#;

    #[test]
    fn synchronising_model() {
        let source = SYNC_MODEL;
        let mut prism = prism_parser::parse_model(source).unwrap();
        let y = prism.variable_manager.get_reference_by_str("y").unwrap();
        let restriction = Expression::var_or_const(y).less_than(Expression::int(2));
        let model = ModelBuilder::new_mdp_builder(&mut prism)
            .with_restricted_state_space(restriction)
            .build();
        assert_eq!(model.states().len(), 3);
        for i in 0..=2 {
            assert_eq!(
                model.state_valuation(StateIndex::from_raw(i)).to_string(),
                format!("x={i},y={i}")
            );
        }
    }

    const CONST_MODEL: &'static str = r#"
        mdp
        const int N=4;
        const int CUTOFF=N - 2;
        module main
            x: [0..N] init 0;
            [] (x<N) -> (x'=x+1);
        endmodule"#;

    #[test]
    fn restriction_with_constant() {
        let source = CONST_MODEL;
        let mut prism = prism_parser::parse_model(source).unwrap();
        let x = prism.variable_manager.get_reference_by_str("x").unwrap();
        let c = prism
            .variable_manager
            .get_reference_by_str("CUTOFF")
            .unwrap();
        let restriction = Expression::var_or_const(x).less_than(Expression::var_or_const(c));
        let model = ModelBuilder::new_mdp_builder(&mut prism)
            .with_restricted_state_space(restriction)
            .build();
        assert_eq!(model.states().len(), 3);
        for i in 0..=2 {
            assert_eq!(
                model.state_valuation(StateIndex::from_raw(i)).to_string(),
                format!("x={i}")
            );
        }
    }

    #[test]
    #[should_panic(
        expected = "Cannot use a state space restriction when building the entire state space of the model"
    )]
    fn conflicting_options() {
        let source = SIMPLE_MODEL;
        let mut prism = prism_parser::parse_model(source).unwrap();
        let x = prism.variable_manager.get_reference_by_str("x").unwrap();
        let restriction = Expression::var_or_const(x).less_than(Expression::int(2));
        let model = ModelBuilder::new_mdp_builder(&mut prism)
            .with_restricted_state_space(restriction)
            .with_full_state_space()
            .build();
    }
}
