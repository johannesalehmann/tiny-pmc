use crate::csr::Csr;
use crate::index::RawIndex;
use crate::{BranchIndex, ChoiceIndex, StateIndex};

pub type StateToChoice<I: RawIndex> = Csr<StateIndex<I>, ChoiceIndex<I>>;
pub type ChoiceToBranch<I: RawIndex> = Csr<ChoiceIndex<I>, BranchIndex<I>>;
