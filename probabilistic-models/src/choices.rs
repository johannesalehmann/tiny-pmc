use crate::{BranchIndex, ChoiceIndex, StateIndex};
use typed_index_collections::{Csr, RawIndex};

pub type StateToChoice<I: RawIndex> = Csr<StateIndex<I>, ChoiceIndex<I>>;
pub type ChoiceToBranch<I: RawIndex> = Csr<ChoiceIndex<I>, BranchIndex<I>>;
