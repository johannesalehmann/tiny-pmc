use typed_index_collections::{Index, To1};

pub struct StateOrdering<SI: Index> {
    pub old_to_new: To1<SI, SI>,
    pub new_to_old: To1<SI, SI>,
}
