use crate::atomic_propositions_builder::AtomicPropositionBuilder;
use crate::initial_states_builder::InitialStatesBuilder;
use crate::initial_states_source::InitialStateSource;
use crate::labels::LabelSource;
use crate::model_type_selection::ModelTypeSelection;
use crate::queries::QueryCollection;
use crate::rewards_builder::RewardsBuilder;
use crate::{ModelBuilder, atomic_propositions_builder, choice_labels};
use prism_model::Span;

impl<
    'a,
    S: Span,
    Q: QueryCollection,
    L: LabelSource,
    IS: InitialStateSource,
    Sel: ModelTypeSelection,
    IB: InitialStatesBuilder<Ix = Sel::Ix>,
    APs: atomic_propositions_builder::AtomicPropositionBuilder<Ix = Sel::Ix>,
    CL: choice_labels::ChoiceLabelBuilder<Ix = Sel::Ix>,
    Rew: RewardsBuilder<Ix = Sel::Ix>,
> ModelBuilder<'a, S, Q, L, IS, Sel, IB, APs, CL, Rew>
{
    pub(crate) fn map_queries<Q2: QueryCollection>(
        self,
        queries: Q2,
    ) -> ModelBuilder<'a, S, Q2, L, IS, Sel, IB, APs, CL, Rew> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries,
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            selection: self.selection,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
            choice_labels: self.choice_labels,
            rewards: self.rewards,
            state_space_restriction: self.state_space_restriction,
        }
    }
    pub(crate) fn map_selection<Sel2: ModelTypeSelection<Ix = Sel::Ix>>(
        self,
        selection: Sel2,
    ) -> ModelBuilder<'a, S, Q, L, IS, Sel2, IB, APs, CL, Rew> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            selection,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
            choice_labels: self.choice_labels,
            rewards: self.rewards,
            state_space_restriction: self.state_space_restriction,
        }
    }
    pub(crate) fn map_queries_with<Q2: QueryCollection>(
        self,
        map: impl FnOnce(Q) -> Q2,
    ) -> ModelBuilder<'a, S, Q2, L, IS, Sel, IB, APs, CL, Rew> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: map(self.queries),
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            selection: self.selection,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
            choice_labels: self.choice_labels,
            rewards: self.rewards,
            state_space_restriction: self.state_space_restriction,
        }
    }
    pub(crate) fn map_labels<L2: LabelSource>(
        self,
        labels: L2,
    ) -> ModelBuilder<'a, S, Q, L2, IS, Sel, IB, APs, CL, Rew> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels,
            initial_state_source: self.initial_state_source,
            selection: self.selection,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
            choice_labels: self.choice_labels,
            rewards: self.rewards,
            state_space_restriction: self.state_space_restriction,
        }
    }

    pub(crate) fn map_initial_state_source<IS2: InitialStateSource>(
        self,
        initial_state_source: IS2,
    ) -> ModelBuilder<'a, S, Q, L, IS2, Sel, IB, APs, CL, Rew> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels: self.labels,
            initial_state_source,
            selection: self.selection,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
            choice_labels: self.choice_labels,
            rewards: self.rewards,
            state_space_restriction: self.state_space_restriction,
        }
    }

    pub(crate) fn map_initial_states_builder<IB2: InitialStatesBuilder<Ix = Sel::Ix>>(
        self,
        initial_states_builder: IB2,
    ) -> ModelBuilder<'a, S, Q, L, IS, Sel, IB2, APs, CL, Rew> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            selection: self.selection,
            base: self.base,
            initial_states_builder,
            atomic_propositions: self.atomic_propositions,
            choice_labels: self.choice_labels,
            rewards: self.rewards,
            state_space_restriction: self.state_space_restriction,
        }
    }

    pub(crate) fn map_atomic_propositions<AP2: AtomicPropositionBuilder<Ix = Sel::Ix>>(
        self,
        atomic_propositions: AP2,
    ) -> ModelBuilder<'a, S, Q, L, IS, Sel, IB, AP2, CL, Rew> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            selection: self.selection,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions,
            choice_labels: self.choice_labels,
            rewards: self.rewards,
            state_space_restriction: self.state_space_restriction,
        }
    }

    pub(crate) fn map_choice_labels<CL2: choice_labels::ChoiceLabelBuilder<Ix = Sel::Ix>>(
        self,
        choice_labels: CL2,
    ) -> ModelBuilder<'a, S, Q, L, IS, Sel, IB, APs, CL2, Rew> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            selection: self.selection,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
            choice_labels,
            rewards: self.rewards,
            state_space_restriction: self.state_space_restriction,
        }
    }

    pub(crate) fn map_rewards<Rew2: RewardsBuilder<Ix = Sel::Ix>>(
        self,
        rewards: Rew2,
    ) -> ModelBuilder<'a, S, Q, L, IS, Sel, IB, APs, CL, Rew2> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            selection: self.selection,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
            choice_labels: self.choice_labels,
            rewards,
            state_space_restriction: self.state_space_restriction,
        }
    }
}
