use super::StateOrdering;
use probabilistic_models::traits::ReadStateSpace;
use typed_index_collections::{Index, To1};

pub struct SubModelConstructionContext<StateIdx: Index, OrderingContext> {
    // For the same model, different sub-models may use different new index types (usually the
    // narrowest-possible type). To support all these in a single buffer, we use usize instead of a
    // specific state index type here.
    pub(super) to_new_state_index: To1<StateIdx, Option<usize>>,
    pub(super) ordering: OrderingContext,
}

impl<StateIdx: Index, OrderingContext> SubModelConstructionContext<StateIdx, OrderingContext> {
    pub fn new<
        M: ReadStateSpace<StateIndex = StateIdx>,
        O: StateOrdering<Context<StateIdx, M::ChoiceIndex> = OrderingContext>,
    >(
        model: &M,
        ordering: &O,
    ) -> Self {
        Self {
            to_new_state_index: To1::with_entries(vec![None; model.states().len()]),
            ordering: ordering.create_context(model),
        }
    }

    pub fn reset<NewSI: Index>(&mut self, to_old_state_index: &To1<NewSI, StateIdx>) {
        for &state in to_old_state_index {
            self.to_new_state_index[state] = None;
        }
    }
}
