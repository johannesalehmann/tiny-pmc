use typed_index_collections::{Csr, Index};

pub type StateToChoice<StateIdx: Index, ChoiceIdx: Index> = Csr<StateIdx, ChoiceIdx>;
pub type ChoiceToBranch<ChoiceIdx: Index, BranchIdx: Index> = Csr<ChoiceIdx, BranchIdx>;
