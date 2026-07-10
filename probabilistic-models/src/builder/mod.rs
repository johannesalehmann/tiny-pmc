mod bases;
pub use bases::BaseModelBuilder;
use bases::MdpBuilder;

mod initial_states;
pub use initial_states::InitialStatesBuilder;
use initial_states::{
    MultipleInitialStatesBuilder, SingleInitialStatesBuilder, UntrackedInitialStatesBuilder,
};

mod atomic_propositions;
use crate::valuations::{
    GetValuationClassIndex, GetValuationData, StandaloneValuation, Valuations,
};
use crate::{Model, StateIndex};
pub use atomic_propositions::AtomicPropositionBuilder;
use atomic_propositions::{AtomicPropositionVectorsBuilder, UntrackedAtomicPropositionBuilder};
use typed_index_collections::Index;

pub struct ModelBuilderBuilder<
    Base: BaseModelBuilder,
    Ini: InitialStatesBuilder,
    APs: AtomicPropositionBuilder,
> {
    base: Base,
    initial_states: Ini,
    atomic_propositions: APs,
}

impl<Base: BaseModelBuilder, Ini: InitialStatesBuilder, APs: AtomicPropositionBuilder>
    ModelBuilderBuilder<Base, Ini, APs>
{
    pub fn new<AnnotationIdx: Index>(
        base: Base,
    ) -> ModelBuilderBuilder<
        Base,
        UntrackedInitialStatesBuilder<Base::StateIdx>,
        UntrackedAtomicPropositionBuilder<AnnotationIdx, Base::StateIdx>,
    > {
        ModelBuilderBuilder {
            base,
            initial_states: UntrackedInitialStatesBuilder::default(),
            atomic_propositions: UntrackedAtomicPropositionBuilder::default(),
        }
    }

    // TODO: Functions to add and remove initial states

    pub fn finish(self) -> ModelBuilder<Base, Ini, APs> {
        ModelBuilder {
            base: self.base,
            initial_states: self.initial_states,
            atomic_propositions: self.atomic_propositions,
        }
    }
}

pub struct ModelBuilder<
    Base: BaseModelBuilder,
    Ini: InitialStatesBuilder,
    APs: AtomicPropositionBuilder,
> {
    pub base: Base,
    pub initial_states: Ini,
    pub atomic_propositions: APs,
}

pub type ModelBuilderOutput<Base, Ini, APs> = Model<
    <Base as BaseModelBuilder>::BaseModel,
    <Ini as InitialStatesBuilder>::InitialStates,
    (),
    (),
    (),
    <APs as AtomicPropositionBuilder>::AtomicPropositions,
    (),
    (),
    <Base as BaseModelBuilder>::Valuation,
>;

impl<
    Base: BaseModelBuilder,
    Ini: InitialStatesBuilder<StateIdx = Base::StateIdx>,
    APs: AtomicPropositionBuilder<StateIdx = Base::StateIdx>,
> ModelBuilder<Base, Ini, APs>
{
    pub fn add_state<
        Val: GetValuationData<Base::ValuationIdx> + GetValuationClassIndex<Base::ClassIdx>,
    >(
        &mut self,
        valuation: Val,
    ) -> Base::StateIdx {
        let index = self.base.add_state(valuation);
        self.initial_states.state_added(index);
        index
    }

    pub fn finish(self) -> ModelBuilderOutput<Base, Ini, APs> {
        let (base, state_valuations) = self.base.into_base_and_valuations();
        Model {
            base,
            initial: self.initial_states.into_initial_states(),
            choice_labels: (),
            branch_labels: (),
            observations: (),
            atomic_propositions: self.atomic_propositions.into_atomic_propositions(),
            rewards: (),
            annotations: (),
            state_valuations,
        }
    }
}
