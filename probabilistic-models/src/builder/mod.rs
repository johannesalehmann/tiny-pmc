mod bases;
pub use bases::BaseModelBuilder;
use bases::MdpBuilder;

mod initial_states;
pub use initial_states::InitialStatesBuilder;
use initial_states::{
    MultipleInitialStatesBuilder, SingleInitialStatesBuilder, UntrackedInitialStatesBuilder,
};

mod atomic_propositions;
pub use atomic_propositions::AtomicPropositionBuilder;
use atomic_propositions::{AtomicPropositionVectorsBuilder, UntrackedAtomicPropositionBuilder};

use crate::valuations::{
    GetValuationClassIndex, GetValuationData, StandaloneValuation, Valuations,
};
use crate::{Model, StateIndex};

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
    pub fn new(
        base: Base,
    ) -> ModelBuilderBuilder<
        Base,
        UntrackedInitialStatesBuilder<Base::Index>,
        UntrackedAtomicPropositionBuilder<Base::Index>,
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
    <Base as BaseModelBuilder>::Index,
    <Base as BaseModelBuilder>::BaseModel,
    <Ini as InitialStatesBuilder>::InitialStates,
    (),
    (),
    (),
    <APs as AtomicPropositionBuilder>::AtomicPropositions,
    (),
    (),
    Valuations<<Base as BaseModelBuilder>::Index, StateIndex<<Base as BaseModelBuilder>::Index>>,
>;

impl<
    Base: BaseModelBuilder,
    Ini: InitialStatesBuilder<Index = Base::Index>,
    APs: AtomicPropositionBuilder<Index = Base::Index>,
> ModelBuilder<Base, Ini, APs>
{
    pub fn add_state<Val: GetValuationData<Base::Index> + GetValuationClassIndex<Base::Index>>(
        &mut self,
        valuation: Val,
    ) -> StateIndex<Base::Index> {
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
            _phantom_data: Default::default(),
        }
    }
}
