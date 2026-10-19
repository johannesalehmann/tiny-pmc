use crate::Model;
use crate::traits::ClearModel;
use typed_index_collections::{Index, To1};

pub trait ReadOwners {
    type OwnerType;
    type StateIdx: Index;
    fn state_owner(&self, state: Self::StateIdx) -> Self::OwnerType;
}

macro_rules! derive_read_owners {
    ($subcomponent:ident) => {
        fn state_owner(&self, state: Self::StateIdx) -> Self::OwnerType {
            self.$subcomponent.state_owner(state)
        }
    };
}
pub(crate) use derive_read_owners;

impl<M: ReadOwners, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> ReadOwners
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type OwnerType = M::OwnerType;
    type StateIdx = M::StateIdx;

    derive_read_owners!(base);
}

impl<T: ReadOwners> ReadOwners for &T {
    type OwnerType = T::OwnerType;
    type StateIdx = T::StateIdx;

    fn state_owner(&self, state: Self::StateIdx) -> Self::OwnerType {
        (**self).state_owner(state)
    }
}
impl<T: ReadOwners> ReadOwners for &mut T {
    type OwnerType = T::OwnerType;
    type StateIdx = T::StateIdx;

    fn state_owner(&self, state: Self::StateIdx) -> Self::OwnerType {
        (**self).state_owner(state)
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
pub struct TurnBased<StateIdx: Index, Owner> {
    pub state_to_owner: To1<StateIdx, Owner>,
}

// Not derived, as that would require `Owner: Default`
impl<StateIdx: Index, Owner> Default for TurnBased<StateIdx, Owner> {
    fn default() -> Self {
        Self {
            state_to_owner: To1::new(),
        }
    }
}

impl ClearModel for SinglePlayer {
    fn clear(&mut self) {}
}

impl<StateIdx: Index, Owner> ClearModel for TurnBased<StateIdx, Owner> {
    fn clear(&mut self) {
        self.state_to_owner.clear();
    }
}
