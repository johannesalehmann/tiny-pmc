pub mod atomic_propositions;

pub mod bases;

pub mod initial_states;
pub mod properties;

use crate::configuration::atomic_propositions::AtomicPropositionVectorsBuilder;
use crate::configuration::bases::MdpBuilder;
use crate::configuration::initial_states::SingleInitialStatesBuilder;
use crate::configuration::properties::ModelOnly;
use atomic_propositions::AtomicPropositionBuilder;
use bases::BaseModelBuilder;
use initial_states::InitialStatesBuilder;
use probabilistic_models::{
    AnnotationEntryIndex, AtomicPropositionIndex, BranchIndex, ChoiceIndex, StateIndex,
    ValuationClassEntryIndex, ValuationClassIndex, ValuationIndex,
};
use properties::QueryCollection;

pub struct ModelBuilder<
    Base: BaseModelBuilder,
    Ini: InitialStatesBuilder,
    APs: AtomicPropositionBuilder,
    Props: QueryCollection,
> {
    base: Base,
    initial_states: Ini,
    atomic_propositions: APs,
    properties: Props,
}

impl
    ModelBuilder<
        MdpBuilder<
            StateIndex<usize>,
            ChoiceIndex<usize>,
            BranchIndex<usize>,
            ValuationClassIndex<u16>,
            ValuationClassEntryIndex<u16>,
            ValuationIndex<usize>,
        >,
        SingleInitialStatesBuilder<StateIndex<usize>>,
        AtomicPropositionVectorsBuilder<
            AtomicPropositionIndex<usize>,
            StateIndex<usize>,
            AnnotationEntryIndex<usize>,
        >,
        ModelOnly,
    >
{
    pub fn new_mdp_builder() -> Self {
        Self {
            base: MdpBuilder::default(),
            initial_states: SingleInitialStatesBuilder::default(),
            atomic_propositions: AtomicPropositionVectorsBuilder::default(),
            properties: ModelOnly::default(),
        }
    }
}
