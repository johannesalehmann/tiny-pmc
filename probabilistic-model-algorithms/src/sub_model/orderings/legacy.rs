use super::StateOrdering;
use crate::mecs::Mecs;
use crate::sccs::Scc;
use crate::sub_model::RewardsSource;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use probabilistic_models::{Ix, StateIndex};
use std::collections::VecDeque;
use typed_index_collections::To1;

pub struct Legacy;

pub struct LegacyContext<I: Ix> {
    visited: To1<StateIndex<I>, bool>,
    open_list: VecDeque<StateIndex<I>>,
}

impl StateOrdering for Legacy {
    type Context<I: Ix> = LegacyContext<I>;

    fn create_context<M: ReadStateSpace>(&self, model: &M) -> Self::Context<M::Ix> {
        LegacyContext {
            visited: To1::with_entries(vec![false; model.states().len()]),
            open_list: VecDeque::new(),
        }
    }

    fn compute_ordering<
        M: ReadStateSpace + ReadPredecessors,
        NewI: Ix,
        Rew: RewardsSource<M::Ix>,
    >(
        &self,
        model: &M,
        scc: Scc<'_, M::Ix>,
        mecs: &Mecs<M::Ix>,
        values: &To1<StateIndex<M::Ix>, f64>,
        rewards: &Rew,
        context: &mut Self::Context<M::Ix>,
        to_old_state_index: &mut To1<StateIndex<NewI>, StateIndex<M::Ix>>,
    ) {
        // Find states that can leave the SCC into a state with a non-zero value or that have a
        // non-zero reward. If an SCC has no such states, all states within it also have value zero,
        // so the sub-model will be empty.
        for state in scc.states() {
            let mut non_zero_exit = has_non_zero_reward(model, rewards, state);
            if !non_zero_exit {
                for successor in model.successors_of_state(state) {
                    if !scc.contains(successor) && values[successor] > 0.0 {
                        non_zero_exit = true;
                        break;
                    }
                }
            }
            if non_zero_exit {
                context.visited[state] = true;
                context.open_list.push_back(state);
            }
        }

        // Perform backwards BFS, visiting predecessors of visited states
        while let Some(state) = context.open_list.pop_front() {
            // States merged into a MEC representative are not added to the sub-model, but they are
            // traversed to visit their predecessors.
            if !mecs.is_merged_away(state) {
                to_old_state_index.add(state);
            }
            for predecessor in model.predecessors_of_state(state) {
                let predecessor_state = model.source_state_of_predecessor(predecessor);
                if !context.visited[predecessor_state] && scc.contains(predecessor_state) {
                    context.visited[predecessor_state] = true;
                    context.open_list.push_back(predecessor_state);
                }
            }
        }

        for state in scc.states() {
            context.visited[state] = false;
        }
    }
}

fn has_non_zero_reward<M: ReadStateSpace, Rew: RewardsSource<M::Ix>>(
    model: &M,
    rewards: &Rew,
    state: StateIndex<M::Ix>,
) -> bool {
    (rewards.has_state_rewards() && rewards.state_reward(state) != 0.0)
        || (rewards.has_choice_rewards()
            && model
                .choices_of_state(state)
                .into_iter()
                .any(|choice| rewards.choice_reward(choice) != 0.0))
}
