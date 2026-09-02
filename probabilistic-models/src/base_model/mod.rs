use crate::traits::ReadStateSpace;
use typed_index_collections::Index;

mod mdp;
mod nonstochastic_game;
mod stochastic_game;
mod transition_system;

pub use mdp::*;

// TODO: While this class is useful as a marker, what is the point of its type parameters? Can't we
//  just refer to the type parameters of ReadStateSpace instead?
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
