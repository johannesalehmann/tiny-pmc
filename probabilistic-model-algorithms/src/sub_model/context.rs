use super::StateOrdering;
use probabilistic_models::traits::ReadStateSpace;
use probabilistic_models::{Ix, StateIndex};
use typed_index_collections::To1;

pub struct SubModelConstructionContext<I: Ix, OrderingContext> {
    // For the same model, different sub-models may use different new index types (usually the
    // narrowest-possible type). To support all these in a single buffer, we use usize instead of a
    // specific state index type here.
    pub(super) to_new_state_index: To1<StateIndex<I>, Option<usize>>,
    pub(super) ordering: OrderingContext,
}

impl<I: Ix, OrderingContext> SubModelConstructionContext<I, OrderingContext> {
    pub fn new<M: ReadStateSpace<Ix = I>, O: StateOrdering<Context<I> = OrderingContext>>(
        model: &M,
        ordering: &O,
    ) -> Self {
        Self {
            to_new_state_index: To1::with_entries(vec![None; model.states().len()]),
            ordering: ordering.create_context(model),
        }
    }

    pub fn reset<NewI: Ix>(&mut self, to_old_state_index: &To1<StateIndex<NewI>, StateIndex<I>>) {
        for &state in to_old_state_index {
            self.to_new_state_index[state] = None;
        }
    }
}
