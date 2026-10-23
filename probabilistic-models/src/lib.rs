pub mod annotations;
mod as_mut;
mod as_ref;
pub mod base_model;
mod component;
pub mod export;
mod info_view;
pub mod initial_states;
mod ix;
pub mod labels;
mod observations;
mod operations;
mod optional;
pub mod predecessors;
pub mod traits;
pub mod valuations;

pub use info_view::PrintLabelInfo;

pub use as_ref::AsRefComponent;
pub use component::{Component, OptionalComponent};
pub use operations::SuccessorOrder;
pub use typed_index_collections;
pub use typed_index_collections::{Index, RawIndex};

pub use ix::{DefaultIx, HasIx, Ix, Sub, Uniform};

use typed_index_collections::index;

// State-sized indices
index!(StateIndex<I: Ix> => I::State);
index!(ValuationIndex<I: Ix> => I::State);
index!(ObservationIndex<I: Ix> => I::State);
// Choice-sized indices
index!(ChoiceIndex<I: Ix> => I::Choice);
// Branch-sized indices
index!(BranchIndex<I: Ix> => I::Branch);
index!(PredecessorIndex<I: Ix> => I::Branch);
// Misc indices
index!(AtomicPropositionIndex<I: Ix> => I::Meta);
index!(RewardIndex<I: Ix> => I::Meta);
index!(AnnotationIndex<I: Ix> => I::Meta);
index!(ActionIndex<I: Ix> => I::Meta);
index!(PlayerIndex<I: Ix> => I::Meta);
index!(ValuationClassIndex<I: Ix> => I::Meta);
index!(ValuationClassEntryIndex<I: Ix> => I::Meta);

pub struct Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> {
    pub base: M,
    pub initial: Ini,
    pub choice_labels: ChLabel,
    pub branch_labels: BrLabel,
    pub observations: Obs,
    pub atomic_propositions: APs,
    pub rewards: Rew,
    pub annotations: Ann,
    pub state_valuations: StateVals, // TODO: Add fields for other valuations
    pub predecessors: Preds,
}

impl<M: HasIx, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> HasIx
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type Ix = M::Ix;
}

impl<M> Model<M, (), (), (), (), (), (), (), (), ()> {
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
            predecessors: (),
        }
    }
}
