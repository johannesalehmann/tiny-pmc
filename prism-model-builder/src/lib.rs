pub use probabilistic_models;
use probabilistic_models::AtomicPropositionIndex;
use probabilistic_models::base_model::BaseModelType;
use probabilistic_models::{ValuationClassEntryIndex, ValuationClassIndex};

pub mod atomic_propositions_builder;
pub mod bases;
pub mod choice_labels;
pub mod constants;
mod expression_context;
pub mod expressions;
pub mod initial_states_builder;
pub mod initial_states_source;
#[cfg(test)]
mod integration_tests;
pub mod labels;
mod map;
pub mod model_type_selection;
mod owners;
pub mod queries;
pub mod rewards_builder;
mod state_builder;
mod state_space_restriction;
mod synchronised_actions;
pub mod variables;

pub use constants::UserProvidedConstValue;

pub use state_builder::UpdatableValuation;

use crate::bases::MdpBuilder;
use crate::expressions::stack_based_expressions::{
    StackBasedExpression, SubExpressionManager, SubExpressionManagerWithCache,
    SubExpressionProvider,
};
use crate::expressions::{TreeWalkingEvaluator, ValuationSource, VariableType};
use crate::model_type_selection::{Auto, ModelTypeSelection};
use crate::owners::{NoOwners, OwnersBuilder, PlayerControl, TurnBasedOwnersBuilder};
use crate::synchronised_actions::SynchronisedActions;
use crate::variables::ModelVariableInfo;
use prism_model::{
    Expression, Identifier, Model, ModelType, RewardsManager, Span, VariableRange,
    VariableReference,
};
use probabilistic_models::Uniform;
use probabilistic_models::base_model::{BaseModel, ChoiceBranchGraph, Mdp};
use probabilistic_models::valuations::Valuations;
use std::collections::{HashMap, VecDeque};

use crate::expression_context::{ExpressionContext, SubExpressionExpressionContext};
use crate::rewards_builder::RewardStructures;
use crate::state_builder::{StateBuilder, StateBuilderVariables};
pub use typed_index_collections::To1;

pub struct ModelBuilder<
    'a,
    S: Span,
    Queries: queries::QueryCollection,
    Labels: labels::LabelSource,
    IniSource: initial_states_source::InitialStateSource,
    Sel: ModelTypeSelection,
    IniBuilder: initial_states_builder::InitialStatesBuilder<Ix = Sel::Ix>,
    APs: atomic_propositions_builder::AtomicPropositionBuilder<Ix = Sel::Ix>,
    CL: choice_labels::ChoiceLabelBuilder<Ix = Sel::Ix>,
    Rew: rewards_builder::RewardsBuilder<Ix = Sel::Ix>,
> {
    model: &'a mut Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
    constants: HashMap<String, UserProvidedConstValue>,

    queries: Queries,
    labels: Labels,
    initial_state_source: IniSource,

    selection: Sel,
    base: MdpBuilder<Sel::Ix>,
    initial_states_builder: IniBuilder,
    atomic_propositions: APs,
    choice_labels: CL,
    rewards: Rew,

    state_space_restriction: Option<Expression<VariableReference, S>>,
}

impl<
    'a,
    S: Span,
    Queries: queries::QueryCollection<Span = S>,
    Labels: labels::LabelSource,
    IniSource: initial_states_source::InitialStateSource,
    Sel: ModelTypeSelection,
    IniBuilder: initial_states_builder::InitialStatesBuilder<Ix = Sel::Ix>,
    APs: atomic_propositions_builder::AtomicPropositionBuilder<Ix = Sel::Ix>,
    CL: choice_labels::ChoiceLabelBuilder<Ix = Sel::Ix>,
    Rew: rewards_builder::RewardsBuilder<Ix = Sel::Ix>,
> ModelBuilder<'a, S, Queries, Labels, IniSource, Sel, IniBuilder, APs, CL, Rew>
{
    pub fn build_and_return_variable_info(
        self,
    ) -> Result<
        (
            Queries::OutputType<
                probabilistic_models::Model<
                    Sel::BaseModel,
                    IniBuilder::InitialStates,
                    CL::ChoiceLabels,
                    (),
                    (),
                    APs::AtomicPropositions,
                    Rew::Rewards,
                    (),
                    Valuations<Sel::Ix>,
                    (),
                >,
                AtomicPropositionIndex<APs::Ix>,
            >,
            ModelVariableInfo<ValuationClassIndex<Sel::Ix>, ValuationClassEntryIndex<Sel::Ix>>,
        ),
        ModelBuildingError,
    > {
        let has_players = !self.model.players.is_empty();
        self.selection.check_players(has_players)?;
        if has_players {
            self.build_with_owners(TurnBasedOwnersBuilder::new, |selection, declared, game| {
                selection.select_with_owners(declared, game)
            })
        } else {
            self.build_with_owners(
                |_| NoOwners::new(),
                |selection, declared, mdp: Mdp<Sel::Ix>| {
                    selection.select_without_owners(declared, mdp)
                },
            )
        }
    }

    // First builds an MDP (and additionally tracks owners using struct `Own`)
    // Then narrows the MDP using `Sel`.
    fn build_with_owners<Own: OwnersBuilder<Ix = Sel::Ix>>(
        mut self,
        create_owners: impl FnOnce(PlayerControl) -> Own,
        select: impl FnOnce(
            &Sel,
            Option<&ModelType<S>>,
            BaseModel<ChoiceBranchGraph<Sel::Ix>, Own::Owners>,
        ) -> Result<Sel::BaseModel, ModelBuildingError>,
    ) -> Result<
        (
            Queries::OutputType<
                probabilistic_models::Model<
                    Sel::BaseModel,
                    IniBuilder::InitialStates,
                    CL::ChoiceLabels,
                    (),
                    (),
                    APs::AtomicPropositions,
                    Rew::Rewards,
                    (),
                    Valuations<Sel::Ix>,
                    (),
                >,
                AtomicPropositionIndex<APs::Ix>,
            >,
            ModelVariableInfo<ValuationClassIndex<Sel::Ix>, ValuationClassEntryIndex<Sel::Ix>>,
        ),
        ModelBuildingError,
    > {
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
        // Reward expressions that are not built would otherwise enlarge the sub-expression cache
        let unused_rewards = (!Rew::stores_rewards())
            .then(|| std::mem::replace(&mut self.model.rewards, RewardsManager::new()));
        let model: Model<_, S, _, _> = self.model_with_sub_expressions(&mut sub_exprs);
        if let Some(rewards) = unused_rewards {
            self.model.rewards = rewards;
        }
        let labels = labels.to_stack_based(&mut sub_exprs, &model.variable_manager);
        let expand_state_condition = match &self.state_space_restriction {
            None => None,
            Some(condition) => {
                let stack =
                    StackBasedExpression::from_expression(condition, &model.variable_manager);
                Some(sub_exprs.add_sub_expression(stack))
            }
        };

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
        let mut owners = create_owners(PlayerControl::new(&model, &synchronising_action)?);
        let reward_structures =
            RewardStructures::new(&model, &synchronising_action, &mut self.rewards);

        let mut state_builder = StateBuilder {
            synchronising_action,
            labels: &labels,
            reward_structures: &reward_structures,
            base: &mut self.base,
            owners: &mut owners,
            initial_states_builder: &mut self.initial_states_builder,
            atomic_propositions: &mut self.atomic_propositions,
            choice_labels: &mut self.choice_labels,
            rewards: &mut self.rewards,
            unlabelled_rewards: Vec::new(),
            unlabelled_rewards_computed: false,
            synchronised_rewards: Vec::new(),
            open_states: VecDeque::new(),
            expand_state_condition,

            variables: StateBuilderVariables {
                info: variable_info,
                model: &model,
                expr_context: &mut expr_context,
            },
        };
        if self.state_space_restriction.is_some()
            && !self
                .initial_state_source
                .compatible_with_state_space_restriction()
        {
            panic!(
                "Cannot use a state space restriction when building the entire state space of the model"
            );
        }
        state_builder.create_initial_states(self.initial_state_source)?;
        state_builder.expand_states()?;
        let variable_info = state_builder.variables.info;

        let (graph, state_valuations) = self.base.into_graph_and_valuations();
        let base = BaseModel {
            graph,
            players: owners.into_owners(),
        };
        let base = select(&self.selection, self.model.model_type.as_ref(), base)?;
        Ok((
            Queries::output(
                probabilistic_models::Model {
                    base,
                    initial: self.initial_states_builder.into_initial_states(),
                    choice_labels: self.choice_labels.into_choice_labels(),
                    branch_labels: (),
                    observations: (),
                    atomic_propositions: self.atomic_propositions.into_atomic_propositions(),
                    rewards: self.rewards.into_rewards(),
                    annotations: (),
                    state_valuations,
                    predecessors: (),
                },
                processed,
            ),
            variable_info,
        ))
    }

    pub fn build(
        self,
    ) -> Result<
        Queries::OutputType<
            probabilistic_models::Model<
                Sel::BaseModel,
                IniBuilder::InitialStates,
                CL::ChoiceLabels,
                (),
                (),
                APs::AtomicPropositions,
                Rew::Rewards,
                (),
                Valuations<Sel::Ix>,
                (),
            >,
            AtomicPropositionIndex<APs::Ix>,
        >,
        ModelBuildingError,
    > {
        let (output, _) = self.build_and_return_variable_info()?;
        Ok(output)
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

impl<'a, S: Span>
    ModelBuilder<
        'a,
        S,
        queries::ModelOnly<S>,
        labels::OnlyNecessary,
        initial_states_source::StartFromInitialStates,
        Auto<Uniform<usize>>,
        initial_states_builder::SingleInitialStatesBuilder<Uniform<usize>>,
        atomic_propositions_builder::AtomicPropositionVectorsBuilder<Uniform<usize>>,
        choice_labels::ActionNameChoiceLabels<Uniform<usize>>,
        rewards_builder::RewardVectorsBuilder<Uniform<usize>>,
    >
{
    pub fn new(
        model: &'a mut Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
    ) -> Self {
        Self {
            model,
            selection: Default::default(),
            base: Default::default(),
            initial_state_source: Default::default(),
            initial_states_builder: Default::default(),
            atomic_propositions: Default::default(),
            queries: Default::default(),
            constants: Default::default(),
            labels: Default::default(),
            choice_labels: Default::default(),
            rewards: Default::default(),
            state_space_restriction: Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum ModelBuildingError {
    /// The model defines players, but the selected model type is not a game.
    ModelHasPlayers,
    /// The selected model type is a game, but the model defines no players.
    ModelHasNoPlayers,
    /// The built model does not fit the requested model type. `narrowest` is the narrowest type
    /// the model fits.
    ModelTypeMismatch {
        requested: BaseModelType,
        narrowest: BaseModelType,
    },
    /// The model type is not supported by the model builder.
    UnsupportedModelType {
        model_type: String,
    },
    UnknownPlayerModule {
        player: String,
        module: String,
    },
    UnknownPlayerAction {
        player: String,
        action: String,
    },
    ModuleControlledByMultiplePlayers {
        module: String,
        players: (String, String),
    },
    ActionControlledByMultiplePlayers {
        action: String,
        players: (String, String),
    },
    /// The choices of a state are controlled by more than one player.
    StateControlledByMultiplePlayers {
        state: usize,
        players: (String, String),
    },
    /// A state has choices, but none of them is controlled by any player.
    StateWithoutOwner {
        state: usize,
    },
}
