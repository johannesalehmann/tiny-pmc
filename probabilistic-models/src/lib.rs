mod annotations;
pub mod builder;
mod choices;
mod initial_states;
pub mod valuations;

use crate::choices::{ChoiceToBranch, StateToChoice};
use std::marker::PhantomData;
use typed_index_collections::{RawIndex, To1, index};

index!(StateIndex);
index!(ChoiceIndex);
index!(BranchIndex);
index!(PlayerIndex);
index!(AnnotationIndex);
index!(AnnotationEntryIndex);
index!(ValuationClassIndex);
index!(ValuationClassEntryIndex);
index!(ValuationIndex);

pub trait BaseModel<I: RawIndex> {
    fn count_states(&self) -> usize;
}

pub struct Mdp<I: RawIndex = u32> {
    state_to_choice: StateToChoice<I>,
    choice_to_branch: ChoiceToBranch<I>,
    branch_probabilities: To1<BranchIndex<I>, f64>,
    branch_targets: To1<BranchIndex<I>, StateIndex<I>>,
}

impl<I: RawIndex> BaseModel<I> for Mdp<I> {
    fn count_states(&self) -> usize {
        self.state_to_choice.count_entries()
    }
}

pub struct Dtmc<I: RawIndex = u32> {
    choice_to_branch: ChoiceToBranch<I>,
    branch_probabilities: To1<BranchIndex<I>, f64>,
    branch_targets: To1<BranchIndex<I>, I>,
}
impl<I: RawIndex> BaseModel<I> for Dtmc<I> {
    fn count_states(&self) -> usize {
        self.choice_to_branch.count_entries()
    }
}

pub struct TransitionSystem<I: RawIndex = u32> {
    state_to_branch: ChoiceToBranch<I>,
    branch_targets: To1<BranchIndex<I>, I>,
}

impl<I: RawIndex> BaseModel<I> for TransitionSystem<I> {
    fn count_states(&self) -> usize {
        self.state_to_branch.count_entries()
    }
}

pub struct Ctmc<I: RawIndex = u32> {
    choice_to_branch: ChoiceToBranch<I>,
    branch_probabilities: To1<BranchIndex<I>, f64>,
    branch_targets: To1<BranchIndex<I>, I>,
    state_to_exit_rate: To1<StateIndex<I>, f64>,
}

impl<I: RawIndex> BaseModel<I> for Ctmc<I> {
    fn count_states(&self) -> usize {
        self.choice_to_branch.count_entries()
    }
}

pub struct Ctmdp<I: RawIndex = u32> {
    state_to_choice: StateToChoice<I>,
    choice_to_branch: ChoiceToBranch<I>,
    branch_probabilities: To1<BranchIndex<I>, f64>,
    branch_targets: To1<BranchIndex<I>, I>,
    state_to_exit_rate: To1<StateIndex<I>, f64>,
}

impl<I: RawIndex> BaseModel<I> for Ctmdp<I> {
    fn count_states(&self) -> usize {
        self.state_to_choice.count_entries()
    }
}

pub type InitialStates<I = u32> = To1<StateIndex<I>, bool>;

pub struct Model<I: RawIndex, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> {
    pub base: M,
    pub initial: Ini,
    pub choice_labels: ChLabel,
    pub branch_labels: BrLabel,
    pub observations: Obs,
    pub atomic_propositions: APs,
    pub rewards: Rew,
    pub annotations: Ann,
    pub state_valuations: StateVals, // TODO: Add fields for other valuations
    _phantom_data: PhantomData<I>,
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
