mod buffer;
pub use buffer::AttractorBuffer;

mod full_region;
pub use full_region::{attractor, attractor_with_buffer};

mod state_included;
pub use state_included::{attractor_contains_state, attractor_contains_state_with_buffer};

use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{ReadOwners, ReadPredecessors, ReadStateSpace};
use typed_index_collections::Index;

// TODO: This algorithm does not work properly on models with stochastic behaviour, but this is not
//  enforced. Perhaps adding a `ReadNonprobabilisticStateSpace` trait would be a good idea?

trait AttractorCondition {
    type StateIdx: Index;
    type Output;

    fn state_attracted(&mut self, index: Self::StateIdx) -> Option<Self::Output>;
    fn result_after_termination(self) -> Self::Output;
}

fn attractor_internal<
    M: ReadStateSpace
        + ReadOwners<StateIdx = <M as ReadStateSpace>::StateIdx, OwnerType = TwoPlayer>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
    R1: Iterator<Item = <M as ReadStateSpace>::StateIdx>,
    C: AttractorCondition<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    region: R1,
    condition: C,
    attracted_player: TwoPlayer,
) -> C::Output {
    let mut buffer = AttractorBuffer::create(model);
    buffer.reset_owner_counts(model, attracted_player);
    attractor_internal_with_buffer(model, region, condition, &mut buffer)
}

// When calling this method, ensure that owner counts in the buffer are up-to-date, either by
// calling .reset_owner_counts(...) on the buffer or by setting them manually.
fn attractor_internal_with_buffer<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
    R1: Iterator<Item = <M as ReadStateSpace>::StateIdx>,
    C: AttractorCondition<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    region: R1,
    mut condition: C,
    buffer: &mut AttractorBuffer<<M as ReadStateSpace>::StateIdx>,
) -> C::Output {
    buffer.open_list.clear();

    for state in region {
        if let Some(result) = condition.state_attracted(state) {
            return result;
        }
        buffer.open_list.push(state);
        buffer.set_value(state, 0);
    }

    while let Some(next) = buffer.open_list.pop() {
        for predecessor in model.predecessors_of_state(next) {
            let from = model
                .state_of_choice(model.choice_of_branch(model.branch_of_predecessor(predecessor)));
            let count = buffer.get_value(from);
            if count > 0 {
                buffer.set_value(from, count - 1);
                if count == 1 {
                    if let Some(result) = condition.state_attracted(from) {
                        return result;
                    }
                    buffer.open_list.push(from);
                }
            }
        }
    }

    condition.result_after_termination()
}
// TODO: This urgently needs tests!
