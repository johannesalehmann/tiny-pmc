use typed_index_collections::{Index, To1};

// Computing dominated-by relations is not yet supported. However, this file already exposes the
//  interface that will be used for this, so that we can architect other algorithms to use it.

pub struct DominatedByRelation<StateIdx: Index> {
    dominated_by: To1<StateIdx, Option<StateIdx>>,
}

impl<StateIdx: Index> DominatedByRelation<StateIdx> {
    pub fn empty() -> Self {
        Self {
            dominated_by: To1::new(),
        }
    }

    pub fn with_entries(dominated_by: To1<StateIdx, Option<StateIdx>>) -> Self {
        Self { dominated_by }
    }

    pub fn dominated_by(&self, state: StateIdx) -> Option<StateIdx> {
        self.dominated_by.get(state).copied().flatten()
    }
}
