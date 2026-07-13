use typed_index_collections::Csr;

pub type StateToChoice<StateIdx, ChoiceIdx> = Csr<StateIdx, ChoiceIdx>;
pub type ChoiceToBranch<ChoiceIdx, BranchIdx> = Csr<ChoiceIdx, BranchIdx>;
