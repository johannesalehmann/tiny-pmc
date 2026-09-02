use crate::traits::ReadStateSpace;
use typed_index_collections::Index;

mod mdp;
mod stochastic_game;

pub use mdp::*;

pub trait BaseModel:
    ReadStateSpace<
        StateIdx = <Self as BaseModel>::StateIndex,
        ChoiceIdx = <Self as BaseModel>::ChoiceIndex,
        BranchIdx = <Self as BaseModel>::BranchIndex,
    >
{
    type StateIndex: Index;
    type ChoiceIndex: Index;
    type BranchIndex: Index;
}
