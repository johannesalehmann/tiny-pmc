use crate::Model;
use typed_index_collections::Index;

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
