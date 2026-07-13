mod bases;
pub use bases::MdpBuilder;
pub use bases::{BaseModelBuilder, ValuationBuilder};

mod initial_states;
pub use initial_states::InitialStatesBuilder;
use initial_states::SingleInitialStatesBuilder;

mod atomic_propositions;
use crate::valuations::{GetValuationClassIndex, GetValuationData};
use crate::{AnnotationEntryIndex, AnnotationIndex, Model};
pub use atomic_propositions::AtomicPropositionBuilder;
use atomic_propositions::AtomicPropositionVectorsBuilder;

pub struct ModelBuilderBuilder<
    Base: BaseModelBuilder,
    Ini: InitialStatesBuilder,
    APs: AtomicPropositionBuilder,
> {
    base: Base,
    initial_states: Ini,
    atomic_propositions: APs,
}

impl<Base: BaseModelBuilder>
    ModelBuilderBuilder<
        Base,
        SingleInitialStatesBuilder<Base::StateIdx>,
        AtomicPropositionVectorsBuilder<
            AnnotationIndex<usize>,
            Base::StateIdx,
            AnnotationEntryIndex<usize>,
        >,
    >
{
    pub fn new(
        base: Base,
    ) -> ModelBuilderBuilder<
        Base,
        SingleInitialStatesBuilder<Base::StateIdx>,
        AtomicPropositionVectorsBuilder<
            AnnotationIndex<usize>,
            Base::StateIdx,
            AnnotationEntryIndex<usize>,
        >,
    > {
        ModelBuilderBuilder {
            base,
            initial_states: SingleInitialStatesBuilder::default(),
            atomic_propositions: AtomicPropositionVectorsBuilder::default(),
        }
    }
}

impl<Base: BaseModelBuilder, Ini: InitialStatesBuilder, APs: AtomicPropositionBuilder>
    ModelBuilderBuilder<Base, Ini, APs>
{
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
    pub fn add_state(&mut self, state_index: Base::StateIdx) {
        let index = self.base.add_state(state_index);
        self.initial_states.state_added(state_index);
        index
    }

    // Call this if you identify a new state (e.g. by following a transition), but are not yet ready
    // to immediately add transitions to it. This already stores the valuation and returns a
    // valuation, but does not yet add it to the states-to-valuations index
    pub fn preregister_state<
        Val: GetValuationData<Base::ValuationIdx> + GetValuationClassIndex<Base::ClassIdx>,
    >(
        &mut self,
        valuation: Val,
    ) -> Base::StateIdx {
        let index = self.base.add_valuation(valuation);
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
