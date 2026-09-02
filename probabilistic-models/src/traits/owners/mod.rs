use typed_index_collections::Index;

pub trait ReadOwners {
    type OwnerType;
    type StateIdx: Index;
    fn state_owner(&self, state: Self::StateIdx) -> Self::OwnerType;
}
