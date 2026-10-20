use crate::traits::ClearModel;
use crate::{HasIx, Ix, Model, StateIndex};
use typed_index_collections::To1;

pub trait ReadOwners: HasIx {
    type OwnerType;
    fn state_owner(&self, state: StateIndex<Self::Ix>) -> Self::OwnerType;
}

macro_rules! derive_read_owners {
    ($subcomponent:ident) => {
        fn state_owner(&self, state: $crate::StateIndex<Self::Ix>) -> Self::OwnerType {
            self.$subcomponent.state_owner(state)
        }
    };
}
pub(crate) use derive_read_owners;

impl<M: ReadOwners, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> ReadOwners
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type OwnerType = M::OwnerType;

    derive_read_owners!(base);
}

impl<T: ReadOwners> ReadOwners for &T {
    type OwnerType = T::OwnerType;

    fn state_owner(&self, state: StateIndex<Self::Ix>) -> Self::OwnerType {
        (**self).state_owner(state)
    }
}
impl<T: ReadOwners> ReadOwners for &mut T {
    type OwnerType = T::OwnerType;

    fn state_owner(&self, state: StateIndex<Self::Ix>) -> Self::OwnerType {
        (**self).state_owner(state)
    }
}

pub trait BuildOwners: ReadOwners {
    // `state` must be the next state index, i.e. the number of states that already have an owner.
    fn add_state_owner(&mut self, state: StateIndex<Self::Ix>, owner: Self::OwnerType);
}

impl<T: BuildOwners> BuildOwners for &mut T {
    fn add_state_owner(&mut self, state: StateIndex<Self::Ix>, owner: Self::OwnerType) {
        (**self).add_state_owner(state, owner)
    }
}

// TODO: It would be nice to just require M: SinglePlayer (avoiding the name clash in some way),
//  instead of M: ReadOwners<OwnerType = SinglePlayer>
// Owner of every state in a model without multiple players, and also the players component of such
// a model. Algorithms can require M: ReadOwners<OwnerType = SinglePlayer> to ensure that no model
// with owners is passed in (which the algorithm might not be able to handle).
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct SinglePlayer;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TwoPlayer {
    Eve,
    Adam,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TurnBased<I: Ix, Owner> {
    pub state_to_owner: To1<StateIndex<I>, Owner>,
}

impl<I: Ix, Owner> HasIx for TurnBased<I, Owner> {
    type Ix = I;
}

// Not derived, as that would require `Owner: Default`
impl<I: Ix, Owner> Default for TurnBased<I, Owner> {
    fn default() -> Self {
        Self {
            state_to_owner: To1::new(),
        }
    }
}

impl<I: Ix, Owner: Copy> ReadOwners for TurnBased<I, Owner> {
    type OwnerType = Owner;

    fn state_owner(&self, state: StateIndex<Self::Ix>) -> Self::OwnerType {
        self.state_to_owner[state]
    }
}

impl<I: Ix, Owner: Copy> BuildOwners for TurnBased<I, Owner> {
    fn add_state_owner(&mut self, state: StateIndex<Self::Ix>, owner: Self::OwnerType) {
        // TODO: On a mismatch, `add_checked` panics with a bare `left == right` assertion that does
        //  not mention state owners. Provide a more helpful message.
        self.state_to_owner.add_checked(state, owner);
    }
}

impl ClearModel for SinglePlayer {
    fn clear(&mut self) {}
}

impl<I: Ix, Owner> ClearModel for TurnBased<I, Owner> {
    fn clear(&mut self) {
        self.state_to_owner.clear();
    }
}
