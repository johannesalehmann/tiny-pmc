use super::StateOrdering;
use crate::mecs::Mecs;
use crate::sccs::Scc;
use crate::sub_model::RewardsSource;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use std::collections::VecDeque;
use typed_index_collections::{Index, To1};

pub struct Legacy;

pub struct LegacyContext<SI: Index> {
    visited: To1<SI, bool>,
    open_list: VecDeque<SI>,
}

impl StateOrdering for Legacy {
    type Context<SI: Index, CI: Index> = LegacyContext<SI>;

    fn create_context<M: ReadStateSpace>(
        &self,
        model: &M,
    ) -> Self::Context<M::StateIndex, M::ChoiceIndex> {
        LegacyContext {
            visited: To1::with_entries(vec![false; model.states().len()]),
            open_list: VecDeque::new(),
        }
    }

    fn compute_ordering<
        M: ReadStateSpace<StateIndex = SI, ChoiceIndex = CI, BranchIndex = BI>
            + ReadPredecessors<StateIdx = SI, ChoiceIdx = CI, BranchIdx = BI>,
        SI: Index,
        CI: Index,
        BI: Index,
        ScI: Index,
        ScEI: Index,
        NewSI: Index,
        Rew: RewardsSource<SI, CI>,
    >(
        &self,
        model: &M,
        scc: Scc<'_, ScI, ScEI, SI>,
        mecs: &Mecs<SI, CI>,
        values: &To1<SI, f64>,
        rewards: &Rew,
        context: &mut Self::Context<SI, CI>,
        to_old_state_index: &mut To1<NewSI, SI>,
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

fn has_non_zero_reward<M: ReadStateSpace, Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>>(
    model: &M,
    rewards: &Rew,
    state: M::StateIndex,
) -> bool {
    (rewards.has_state_rewards() && rewards.state_reward(state) != 0.0)
        || (rewards.has_choice_rewards()
            && model
                .choices_of_state(state)
                .into_iter()
                .any(|choice| rewards.choice_reward(choice) != 0.0))
}
