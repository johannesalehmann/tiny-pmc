pub use probabilistic_models;

pub mod atomic_propositions_builder;
pub mod bases;
pub mod choice_labels;
pub mod constants;
mod expression_context;
pub mod expressions;
pub mod initial_states_builder;
pub mod initial_states_source;
pub mod labels;
mod map;
pub mod queries;
mod state_builder;
mod synchronised_actions;
pub mod variables;

pub use constants::UserProvidedConstValue;

pub use state_builder::UpdatableValuation;

use crate::expressions::stack_based_expressions::{
    StackBasedExpression, SubExpressionManager, SubExpressionManagerWithCache,
    SubExpressionProvider,
};
use crate::expressions::{TreeWalkingEvaluator, ValuationSource, VariableType};
use crate::synchronised_actions::SynchronisedActions;
use crate::variables::ModelVariableInfo;
use prism_model::{Expression, Identifier, Model, Span, VariableRange, VariableReference};
use std::collections::{HashMap, VecDeque};

use crate::expression_context::{ExpressionContext, SubExpressionExpressionContext};
use crate::state_builder::{StateBuilder, StateBuilderVariables};
pub use typed_index_collections::To1;

pub struct ModelBuilder<
    'a,
    S: Span,
    Queries: queries::QueryCollection,
    Labels: labels::LabelSource,
    IniSource: initial_states_source::InitialStateSource,
    Base: bases::BaseModelBuilder,
    IniBuilder: initial_states_builder::InitialStatesBuilder<StateIdx = Base::StateIdx>,
    APs: atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = Base::StateIdx>,
    CL: choice_labels::ChoiceLabelBuilder<ChoiceIdx = Base::ChoiceIdx>,
> {
    model: &'a mut Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
    constants: HashMap<String, UserProvidedConstValue>,

    queries: Queries,
    labels: Labels,
    initial_state_source: IniSource,

    base: Base,
    initial_states_builder: IniBuilder,
    atomic_propositions: APs,
    choice_labels: CL,
}

impl<
    'a,
    S: Span,
    Queries: queries::QueryCollection<Span = S>,
    Labels: labels::LabelSource,
    IniSource: initial_states_source::InitialStateSource,
    Base: bases::BaseModelBuilder,
    IniBuilder: initial_states_builder::InitialStatesBuilder<StateIdx = Base::StateIdx>,
    APs: atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = Base::StateIdx>,
    CL: choice_labels::ChoiceLabelBuilder<ChoiceIdx = Base::ChoiceIdx>,
> ModelBuilder<'a, S, Queries, Labels, IniSource, Base, IniBuilder, APs, CL>
{
    pub fn build_and_return_variable_info(
        mut self,
    ) -> (
        Queries::OutputType<
            probabilistic_models::Model<
                Base::BaseModel,
                IniBuilder::InitialStates,
                CL::ChoiceLabels,
                (),
                (),
                APs::AtomicPropositions,
                (),
                (),
                Base::Valuation,
                (),
            >,
            APs::APIdx,
        >,
        ModelVariableInfo<Base::ClassIdx, Base::ClassEntryIdx>,
    ) {
        self.model.replace_empty_updates_with_identity_update();

        let mut labels = self.labels.extract_labels(self.model);

        let variable_info = ModelVariableInfo::new(
            &self.model,
            &self.constants,
            &mut TreeWalkingEvaluator::new(),
            self.base.valuation_builder_mut(),
        )
        .unwrap();

        let processed = self.queries.process_queries(
            &self.model.labels,
            &mut labels,
            &mut TreeWalkingEvaluator::new(),
            &variable_info.get_const_only_valuation_source(),
        );

        let mut sub_exprs = SubExpressionManager::new();
        let model: Model<_, S, _, _> = self.model_with_sub_expressions(&mut sub_exprs);
        let labels = labels.to_stack_based(&mut sub_exprs, &model.variable_manager);

        for (index, (name, _)) in labels.into_iter().enumerate() {
            let new_index = self
                .atomic_propositions
                .register_atomic_proposition(name.to_string());
            assert_eq!(index, new_index);
        }

        let mut expr_cache = SubExpressionManagerWithCache::new(sub_exprs);
        expr_cache.manager.optimise_expressions(&variable_info);
        let context = expr_cache.create_context();
        let mut expr_context = SubExpressionExpressionContext {
            sub_expressions: &expr_cache,
            context,
        };

        let synchronising_action = SynchronisedActions::from_prism(&model);

        let mut state_builder = StateBuilder {
            synchronising_action,
            labels: &labels,
            base: &mut self.base,
            initial_states_builder: &mut self.initial_states_builder,
            atomic_propositions: &mut self.atomic_propositions,
            choice_labels: &mut self.choice_labels,
            open_states: VecDeque::new(),

            variables: StateBuilderVariables {
                info: variable_info,
                model: &model,
                expr_context: &mut expr_context,
            },
        };
        state_builder
            .create_initial_states(self.initial_state_source)
            .unwrap();
        state_builder.expand_states().unwrap();
        let variable_info = state_builder.variables.info;

        let (base, state_valuations) = self.base.into_base_and_valuations();
        (
            Queries::output(
                probabilistic_models::Model {
                    base,
                    initial: self.initial_states_builder.into_initial_states(),
                    choice_labels: self.choice_labels.into_choice_labels(),
                    branch_labels: (),
                    observations: (),
                    atomic_propositions: self.atomic_propositions.into_atomic_propositions(),
                    rewards: (),
                    annotations: (),
                    state_valuations,
                    predecessors: (),
                },
                processed,
            ),
            variable_info,
        )
    }

    pub fn build(
        self,
    ) -> Queries::OutputType<
        probabilistic_models::Model<
            Base::BaseModel,
            IniBuilder::InitialStates,
            CL::ChoiceLabels,
            (),
            (),
            APs::AtomicPropositions,
            (),
            (),
            Base::Valuation,
            (),
        >,
        APs::APIdx,
    > {
        let (output, _) = self.build_and_return_variable_info();
        output
    }

    fn model_with_sub_expressions(
        &self,
        sub_expressions: &mut SubExpressionManager<VariableReference>,
    ) -> Model<VariableReference, S, usize, Identifier<S>> {
        self.model.map_expressions_cloned(|e| {
            let stack = StackBasedExpression::from_expression(e, &self.model.variable_manager);
            let sub_expression_index = sub_expressions.add_sub_expression(stack);
            sub_expression_index
        })
    }
}

#[derive(Debug)]
pub enum ModelBuildingError {}
