use typed_index_collections::Index;

pub struct States<StateIndex: Index> {
    size: StateIndex,
}

pub struct Choices<ChoiceIndex: Index> {
    size: ChoiceIndex,
}

pub struct Branches<BranchIndex: Index> {
    size: BranchIndex,
}

pub struct ChoiceRange<ChoiceIndex: Index> {
    start: ChoiceIndex,
    end: ChoiceIndex,
}
pub struct BranchRange<BranchIndex: Index> {
    start: BranchIndex,
    end: BranchIndex,
}
