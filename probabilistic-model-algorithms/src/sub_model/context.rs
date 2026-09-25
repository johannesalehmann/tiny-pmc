use probabilistic_models::traits::ReadStateSpace;
use std::collections::VecDeque;
use typed_index_collections::{Index, To1};

pub struct SubModelConstructionContext<StateIdx: Index> {
    // For the same model, different sub-models may use different new index types (usually the
    // narrowest-possible type). To support all these in a single buffer, we use usize instead of a
    // specific state index type here.
    pub(super) to_new_state_index: To1<StateIdx, Option<usize>>,
    // Which states were visited in the BFS that is used to determine the order within the sub-model
    pub(super) visited: To1<StateIdx, bool>,
    pub(super) visitation_order: Vec<StateIdx>,
    pub(super) visited_open_list: VecDeque<StateIdx>,
}

impl<StateIdx: Index> SubModelConstructionContext<StateIdx> {
    pub fn new<M: ReadStateSpace<StateIndex = StateIdx>>(model: &M) -> Self {
        Self {
            to_new_state_index: To1::with_entries(vec![None; model.states().len()]),
            visited: To1::with_entries(vec![false; model.states().len()]),
            visitation_order: Vec::new(),
            visited_open_list: VecDeque::new(),
        }
    }

    pub fn reset<NewSI: Index>(&mut self, to_old_state_index: &To1<NewSI, StateIdx>) {
        // TODO: `visited` is not reset for dominated and merged-away states visited by the BFS.
        //  This becomes problematic if the context is reused for multiple models (for a single
        //  model, the SCCs never overlap).
        for &state in to_old_state_index {
            self.to_new_state_index[state] = None;
            self.visited[state] = false;
        }
        self.visitation_order.clear();
        self.visited_open_list.clear();
    }
}
