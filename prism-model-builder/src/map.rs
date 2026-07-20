use crate::atomic_propositions_builder::AtomicPropositionBuilder;
use crate::bases::BaseModelBuilder;
use crate::initial_states_builder::InitialStatesBuilder;
use crate::initial_states_source::InitialStateSource;
use crate::labels::LabelSource;
use crate::queries::QueryCollection;
use crate::{ModelBuilder, atomic_propositions_builder};
use prism_model::Span;

impl<
    'a,
    S: Span,
    Q: QueryCollection,
    L: LabelSource,
    IS: InitialStateSource,
    B: BaseModelBuilder,
    IB: InitialStatesBuilder<StateIdx = B::StateIdx>,
    APs: atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
> ModelBuilder<'a, S, Q, L, IS, B, IB, APs>
{
    pub(crate) fn map_queries<Q2: QueryCollection>(
        self,
        queries: Q2,
    ) -> ModelBuilder<'a, S, Q2, L, IS, B, IB, APs> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries,
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
        }
    }
    pub(crate) fn map_queries_with<Q2: QueryCollection>(
        self,
        map: impl FnOnce(Q) -> Q2,
    ) -> ModelBuilder<'a, S, Q2, L, IS, B, IB, APs> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: map(self.queries),
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
        }
    }
    pub(crate) fn map_labels<L2: LabelSource>(
        self,
        labels: L2,
    ) -> ModelBuilder<'a, S, Q, L2, IS, B, IB, APs> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels,
            initial_state_source: self.initial_state_source,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
        }
    }

    pub(crate) fn map_initial_state_source<IS2: InitialStateSource>(
        self,
        initial_state_source: IS2,
    ) -> ModelBuilder<'a, S, Q, L, IS2, B, IB, APs> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels: self.labels,
            initial_state_source,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions: self.atomic_propositions,
        }
    }

    pub(crate) fn map_initial_states_builder<IB2: InitialStatesBuilder<StateIdx = B::StateIdx>>(
        self,
        initial_states_builder: IB2,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB2, APs> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            base: self.base,
            initial_states_builder,
            atomic_propositions: self.atomic_propositions,
        }
    }

    pub(crate) fn map_atomic_propositions<AP2: AtomicPropositionBuilder<StateIdx = B::StateIdx>>(
        self,
        atomic_propositions: AP2,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, AP2> {
        ModelBuilder {
            model: self.model,
            constants: self.constants,
            queries: self.queries,
            labels: self.labels,
            initial_state_source: self.initial_state_source,
            base: self.base,
            initial_states_builder: self.initial_states_builder,
            atomic_propositions,
        }
    }
}
