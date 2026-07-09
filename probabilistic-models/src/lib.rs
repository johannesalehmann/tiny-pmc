mod annotations;
mod builder;
mod choices;
mod csr;
mod index;
mod initial_states;
mod named_collection;
mod to1;
mod valuations;

use crate::choices::{ChoiceToBranch, StateToChoice};
use crate::to1::To1;
pub use index::{
    AnnotationEntryIndex, AnnotationIndex, BranchIndex, ChoiceIndex, Index, PlayerIndex, RawIndex,
    StateIndex, ValuationClassEntryIndex, ValuationClassIndex, ValuationIndex,
};
use std::marker::PhantomData;

pub trait BaseModel<I: RawIndex> {}

pub struct Mdp<I: RawIndex = u32> {
    state_to_choice: StateToChoice<I>,
    choice_to_branch: ChoiceToBranch<I>,
    branch_probabilities: To1<BranchIndex<I>, f64>,
    branch_targets: To1<BranchIndex<I>, StateIndex<I>>,
}

impl<I: RawIndex> BaseModel<I> for Mdp<I> {}

pub struct Dtmc<I: RawIndex = u32> {
    choice_to_branch: ChoiceToBranch<I>,
    branch_probabilities: To1<BranchIndex<I>, f64>,
    branch_targets: To1<BranchIndex<I>, I>,
}
impl<I: RawIndex> BaseModel<I> for Dtmc<I> {}

pub struct TransitionSystem<I: RawIndex = u32> {
    state_to_branch: ChoiceToBranch<I>,
    branch_targets: To1<BranchIndex<I>, I>,
}

impl<I: RawIndex> BaseModel<I> for TransitionSystem<I> {}

pub struct Ctmc<I: RawIndex = u32> {
    choice_to_branch: ChoiceToBranch<I>,
    branch_probabilities: To1<BranchIndex<I>, f64>,
    branch_targets: To1<BranchIndex<I>, I>,
    state_to_exit_rate: To1<StateIndex<I>, f64>,
}

impl<I: RawIndex> BaseModel<I> for Ctmc<I> {}

pub struct Ctmdp<I: RawIndex = u32> {
    state_to_choice: StateToChoice<I>,
    choice_to_branch: ChoiceToBranch<I>,
    branch_probabilities: To1<BranchIndex<I>, f64>,
    branch_targets: To1<BranchIndex<I>, I>,
    state_to_exit_rate: To1<StateIndex<I>, f64>,
}

impl<I: RawIndex> BaseModel<I> for Ctmdp<I> {}

pub type InitialStates<I: RawIndex = u32> = To1<StateIndex<I>, bool>;

pub struct Model<I: RawIndex, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> {
    base: M,
    initial: Ini,
    choice_labels: ChLabel,
    branch_labels: BrLabel,
    observations: Obs,
    atomic_propositions: APs,
    rewards: Rew,
    annotations: Ann,
    state_valuations: StateVals, // TODO: Support other valuations
    _phantom_data: PhantomData<(I)>,
}

impl<I: RawIndex, M: BaseModel<I>> Model<I, M, (), (), (), (), (), (), (), ()>
where
    M: BaseModel<I>,
{
    pub fn new(base: M) -> Self {
        Self {
            base,
            initial: (),
            choice_labels: (),
            branch_labels: (),
            observations: (),
            atomic_propositions: (),
            rewards: (),
            annotations: (),
            state_valuations: (),
            _phantom_data: PhantomData,
        }
    }
}
