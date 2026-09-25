mod context;
pub use context::SubModelConstructionContext;

mod sub_model_rewards;
pub use sub_model_rewards::{RewardsSource, StateAndChoiceRewards};

use crate::dominated_by::DominatedByRelation;
use crate::mecs::Mecs;
use crate::sccs::Scc;
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use typed_index_collections::{Index, RawIndex, To1};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SubModelOrder {
    BackToFront,
    FrontToBack,
    AttractorStyle,
    Legacy,
}

pub struct SubModel<StateIdx: Index, NewSI: Index, NewCI: Index, NewBI: Index> {
    pub mdp: Mdp<NewSI, NewCI, NewBI>,
    /// The value that external states (i.e. those outside the SCC) contribute to a choice.
    pub choice_exit_values: To1<NewCI, f64>,
    pub to_old_state_index: To1<NewSI, StateIdx>,
}

impl<StateIdx: Index, NewSI: Index, NewCI: Index, NewBI: Index>
    SubModel<StateIdx, NewSI, NewCI, NewBI>
{
    pub fn empty() -> Self {
        Self {
            mdp: Mdp::default(),
            choice_exit_values: To1::new(),
            to_old_state_index: To1::new(),
        }
    }

    pub fn from_active_states<
        M: ReadStateSpace<StateIndex = StateIdx> + ReadPredecessors<StateIdx = StateIdx>,
    >(
        model: &M,
        dominated_by: &DominatedByRelation<M::StateIndex>,
        s0: &To1<M::StateIndex, bool>,
        s1: &To1<M::StateIndex, bool>,
    ) -> Self {
        let _ = (model, dominated_by, s0, s1);
        todo!()
    }

    pub fn from_scc<
        M: ReadStateSpace<StateIndex = StateIdx> + ReadPredecessors<StateIdx = StateIdx>,
        ScI: Index,
        ScEI: Index,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
    >(
        model: &M,
        scc: Scc<'_, ScI, ScEI, M::StateIndex>,
        dominated_by: &DominatedByRelation<M::StateIndex>,
        mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
        values: &To1<M::StateIndex, f64>,
        rewards: &Rew,
        order: SubModelOrder,
    ) -> Self {
        let mut context = SubModelConstructionContext::new(model);
        let mut sub_model = SubModel::empty();
        sub_model.rebuild_from_scc(
            model,
            scc,
            dominated_by,
            mecs,
            values,
            rewards,
            order,
            &mut context,
        );
        sub_model
    }

    pub fn rebuild_from_scc<
        M: ReadStateSpace<StateIndex = StateIdx> + ReadPredecessors<StateIdx = StateIdx>,
        ScI: Index,
        ScEI: Index,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
    >(
        &mut self,
        model: &M,
        scc: Scc<'_, ScI, ScEI, M::StateIndex>,
        dominated_by: &DominatedByRelation<M::StateIndex>,
        mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
        values: &To1<M::StateIndex, f64>,
        rewards: &Rew,
        order: SubModelOrder,
        context: &mut SubModelConstructionContext<M::StateIndex>,
    ) {
        compute_order(
            model,
            scc,
            dominated_by,
            mecs,
            values,
            rewards,
            order,
            context,
            &mut self.to_old_state_index,
        );

        self.mdp.clear();
        self.choice_exit_values.clear();

        for (new_state, &state) in self.to_old_state_index.enumerate() {
            self.mdp.add_state(new_state);
            for member in mecs.states_merged_into(state) {
                let state_reward = if rewards.has_state_rewards() {
                    rewards.state_reward(member)
                } else {
                    0.0
                };
                for choice in model.choices_of_state(member) {
                    if mecs.is_internal_choice(choice) {
                        continue;
                    }
                    let choice_index = self.mdp.add_choice();
                    let mut to_self = 0.0;
                    let mut exit_value = 0.0;
                    for branch in model.branches_of_choice(choice) {
                        let mut destination = model.branch_destination(branch);
                        if let Some(representative) = mecs.representative(destination) {
                            destination = representative;
                        }
                        if let Some(dominating_state) = dominated_by.dominated_by(destination) {
                            destination = dominating_state;
                        }
                        let p = model.branch_probability(branch);

                        if destination == state {
                            to_self += p;
                        } else if let Some(target) = context.to_new_state_index[destination] {
                            // We first add the actual probability. After the loop, we then scale
                            // the probability to account for removed self loops.
                            let target = NewSI::from_raw(NewSI::RawType::from_usize(target));
                            self.mdp.add_branch(p, target);
                        } else {
                            exit_value += p * values[destination];
                        }
                    }

                    let reward = if rewards.has_choice_rewards() {
                        state_reward + rewards.choice_reward(choice)
                    } else {
                        state_reward
                    };

                    // Preserve actions that are just a self-loop (relevant at least for minimum
                    //  reachability probability).
                    if to_self == 1.0 {
                        self.mdp.add_branch(1.0, new_state);
                        assert_eq!(exit_value, 0.0);
                        self.choice_exit_values.add_checked(choice_index, reward);
                    } else {
                        let scale_factor = 1.0 / (1.0 - to_self);
                        for branch in self.mdp.branches_of_choice(choice_index) {
                            self.mdp.branch_probabilities[branch] *= scale_factor;
                        }
                        self.choice_exit_values
                            .add_checked(choice_index, (exit_value + reward) * scale_factor);
                    }
                }
            }
        }

        context.reset(&self.to_old_state_index);
    }
}

fn compute_order<
    M: ReadStateSpace + ReadPredecessors<StateIdx = M::StateIndex>,
    ScI: Index,
    ScEI: Index,
    NewSI: Index,
    Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
>(
    model: &M,
    scc: Scc<'_, ScI, ScEI, M::StateIndex>,
    dominated_by: &DominatedByRelation<M::StateIndex>,
    mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
    values: &To1<M::StateIndex, f64>,
    rewards: &Rew,
    order: SubModelOrder,
    context: &mut SubModelConstructionContext<M::StateIndex>,
    to_old_state_index: &mut To1<NewSI, M::StateIndex>,
) {
    match order {
        SubModelOrder::BackToFront | SubModelOrder::FrontToBack => compute_index_based_order(
            scc,
            dominated_by,
            mecs,
            order == SubModelOrder::BackToFront,
            context,
            to_old_state_index,
        ),
        SubModelOrder::AttractorStyle => {
            todo!()
        }
        SubModelOrder::Legacy => compute_order_bfs(
            model,
            scc,
            dominated_by,
            mecs,
            values,
            rewards,
            context,
            to_old_state_index,
        ),
    }
}

fn compute_index_based_order<SI: Index, CI: Index, ScI: Index, ScEI: Index, NewSI: Index>(
    scc: Scc<'_, ScI, ScEI, SI>,
    dominated_by: &DominatedByRelation<SI>,
    mecs: &Mecs<SI, CI>,
    reverse: bool,
    context: &mut SubModelConstructionContext<SI>,
    to_old_state_index: &mut To1<NewSI, SI>,
) {
    to_old_state_index.clear();
    for state in scc.states() {
        if dominated_by.dominated_by(state).is_none() && !mecs.is_merged_away(state) {
            to_old_state_index.add(state);
        }
    }
    let states = to_old_state_index.entries_mut();
    states.sort_unstable();
    if reverse {
        states.reverse();
    }
    for (new_index, &state) in to_old_state_index.enumerate() {
        context.to_new_state_index[state] = Some(new_index.raw().as_usize());
    }
}

fn compute_order_bfs<
    M: ReadStateSpace + ReadPredecessors<StateIdx = M::StateIndex>,
    ScI: Index,
    ScEI: Index,
    NewSI: Index,
    Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
>(
    model: &M,
    scc: Scc<'_, ScI, ScEI, M::StateIndex>,
    dominated_by: &DominatedByRelation<M::StateIndex>,
    mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
    values: &To1<M::StateIndex, f64>,
    rewards: &Rew,
    context: &mut SubModelConstructionContext<M::StateIndex>,
    to_old_state_index: &mut To1<NewSI, M::StateIndex>,
) {
    to_old_state_index.clear();
    // Find states that can leave the SCC into a state with a non-zero value or that have a non-zero
    // reward. If an SCC has no such states, all states within it also have value zero, so the
    // sub-model will be empty.
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
            context.visited_open_list.push_back(state);
        }
    }

    // Perform backwards BFS, visiting predecessors of visited states
    while let Some(state) = context.visited_open_list.pop_front() {
        context.visitation_order.push(state);
        // Dominated states and states merged into a MEC representative are not added to the
        // sub-model, but they are traversed to visit their predecessors.
        if dominated_by.dominated_by(state).is_none() && !mecs.is_merged_away(state) {
            let new_index = to_old_state_index.add(state);
            context.to_new_state_index[state] = Some(new_index.raw().as_usize());
        }
        for predecessor in model.predecessors_of_state(state) {
            let predecessor_state = model.source_state_of_predecessor(predecessor);
            if !context.visited[predecessor_state] && scc.contains(predecessor_state) {
                context.visited[predecessor_state] = true;
                context.visited_open_list.push_back(predecessor_state);
            }
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

#[cfg(test)]
mod tests {
    use super::{SubModel, SubModelConstructionContext, SubModelOrder};
    use crate::dominated_by::DominatedByRelation;
    use crate::mecs::Mecs;
    use crate::sccs::{ExcludeStatesAndChoices, SccEntryIndex, SccIndex, Sccs};
    use crate::value_iteration::precomputed_states::S0S1;
    use probabilistic_models::mdp;
    use probabilistic_models::{BranchIndex, ChoiceIndex, Model, PredecessorIndex, StateIndex};
    use typed_index_collections::{Csr, Index, To1};

    #[test]
    fn loop_removal_and_rescaling() {
        // States 0 and 1 form an SCC, state 2 is the goal state.
        mdp!(mdp = {
            s0 -> 0.25: s0 & 0.25: s1 & 0.5: s2,
            s1 -> 1.0: s0,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(
            &model,
            &S0S1::new(
                To1::with_entries(vec![false, false, false]),
                To1::with_entries(vec![false, false, true]),
            ),
        );
        let values = To1::with_entries(vec![0.0, 0.0, 1.0]);
        let mut context = SubModelConstructionContext::new(&model);

        let mut sub_model: SubModel<_, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> =
            SubModel::empty();
        sub_model.rebuild_from_scc(
            &model,
            sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
            &DominatedByRelation::empty(),
            &Mecs::empty(),
            &values,
            &(),
            SubModelOrder::BackToFront,
            &mut context,
        );

        assert_eq!(
            sub_model.to_old_state_index,
            To1::with_entries(vec![StateIndex::from_raw(1), StateIndex::from_raw(0)])
        );
        assert_eq!(
            sub_model.mdp.state_to_choice,
            Csr::with_entries(vec![ChoiceIndex::from_raw(1), ChoiceIndex::from_raw(2)])
        );
        assert_eq!(
            sub_model.mdp.choice_to_branch,
            Csr::with_entries(vec![BranchIndex::from_raw(1), BranchIndex::from_raw(2)])
        );
        assert_eq!(
            sub_model.mdp.branch_destinations,
            To1::with_entries(vec![StateIndex::from_raw(1), StateIndex::from_raw(0)])
        );
        assert_eq!(
            sub_model.mdp.branch_probabilities,
            To1::with_entries(vec![1.0, 0.25 * (1.0 / 0.75)])
        );
        assert_eq!(
            sub_model.choice_exit_values,
            To1::with_entries(vec![0.0, 0.5 * 1.0 * (1.0 / 0.75)])
        );
    }

    #[test]
    fn external_branch_values() {
        // State 0 forms an SCC of its own and leaves it into state 1 and into the goal state 2.
        mdp!(mdp = {
            s0 -> 0.5: s0 & 0.25: s1 & 0.25: s2,
            s1 -> 1.0: s1,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(
            &model,
            &S0S1::new(
                To1::with_entries(vec![false, false, false]),
                To1::with_entries(vec![false, false, true]),
            ),
        );
        let values = To1::with_entries(vec![0.0, 0.6, 1.0]);
        let mut context = SubModelConstructionContext::new(&model);

        let mut sub_model: SubModel<_, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> =
            SubModel::empty();
        sub_model.rebuild_from_scc(
            &model,
            sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
            &DominatedByRelation::empty(),
            &Mecs::empty(),
            &values,
            &(),
            SubModelOrder::BackToFront,
            &mut context,
        );

        assert_eq!(
            sub_model.to_old_state_index,
            To1::with_entries(vec![StateIndex::from_raw(0)])
        );
        assert_eq!(
            sub_model.mdp.state_to_choice,
            Csr::with_entries(vec![ChoiceIndex::from_raw(1)])
        );
        // Both remaining branches leave the sub-model, so the only choice has no branches left.
        assert_eq!(
            sub_model.mdp.choice_to_branch,
            Csr::with_entries(vec![BranchIndex::from_raw(0)])
        );
        assert_eq!(
            sub_model.choice_exit_values,
            To1::with_entries(vec![(0.25 * 0.6 + 0.25 * 1.0) * (1.0 / 0.5)])
        );
    }

    #[test]
    fn dominating_states() {
        // States 0 and 1 form an SCC, state 2 is the goal state.
        mdp!(mdp = {
            s0 -> 0.5: s0 & 0.5: s1,
            s0 -> 0.25: s0 & 0.25: s1 & 0.5: s2,
            s1 -> 1.0: s0,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(
            &model,
            &S0S1::new(
                To1::with_entries(vec![false, false, false]),
                To1::with_entries(vec![false, false, true]),
            ),
        );
        let values = To1::with_entries(vec![0.0, 0.0, 1.0]);
        let dominated_by = DominatedByRelation::with_entries(To1::with_entries(vec![
            None,
            Some(StateIndex::from_raw(0)),
            None,
        ]));
        let mut context = SubModelConstructionContext::new(&model);

        let mut sub_model: SubModel<_, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> =
            SubModel::empty();
        sub_model.rebuild_from_scc(
            &model,
            sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
            &dominated_by,
            &Mecs::empty(),
            &values,
            &(),
            SubModelOrder::BackToFront,
            &mut context,
        );

        // State 1 is dominated by state 0 and thus neither contributes a state nor its choice.
        assert_eq!(
            sub_model.to_old_state_index,
            To1::with_entries(vec![StateIndex::from_raw(0)])
        );
        assert_eq!(
            sub_model.mdp.state_to_choice,
            Csr::with_entries(vec![ChoiceIndex::from_raw(2)])
        );
        // The first action turns into a p=1 self loop. This is not removed by the self-loop
        // removal (otherwise, it would produce incorrect probabilities for minimal reachability).
        // The second action creates a p=0.5 self loop, with the other 0.5 leaving the sub-model.
        // Thus, that action has no branches.
        assert_eq!(
            sub_model.mdp.choice_to_branch,
            Csr::with_entries(vec![BranchIndex::from_raw(1), BranchIndex::from_raw(1)])
        );
        assert_eq!(
            sub_model.choice_exit_values,
            To1::with_entries(vec![0.0, 0.5 * 1.0 * (1.0 / 0.5)])
        );
    }

    #[test]
    fn context_reuse() {
        // State 0 leaves its own SCC into the SCC of state 1, which is built first. State 1
        // leaves its SCC into the goal state 2.
        mdp!(mdp = {
            s0 -> 0.5: s0 & 0.5: s1,
            s1 -> 0.5: s1 & 0.5: s2,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(
            &model,
            &S0S1::new(
                To1::with_entries(vec![false, false, false]),
                To1::with_entries(vec![false, false, true]),
            ),
        );
        let values = To1::with_entries(vec![0.0, 0.6, 1.0]);
        let mut context = SubModelConstructionContext::new(&model);

        let mut sub_models: Vec<
            SubModel<StateIndex<usize>, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>,
        > = Vec::new();
        for scc in sccs.reverse_topological_ordering() {
            let mut sub_model: SubModel<
                _,
                StateIndex<usize>,
                ChoiceIndex<usize>,
                BranchIndex<usize>,
            > = SubModel::empty();
            sub_model.rebuild_from_scc(
                &model,
                scc,
                &DominatedByRelation::empty(),
                &Mecs::empty(),
                &values,
                &(),
                SubModelOrder::BackToFront,
                &mut context,
            );
            sub_models.push(sub_model);
        }

        assert_eq!(
            sub_models[0].to_old_state_index,
            To1::with_entries(vec![StateIndex::from_raw(1)])
        );
        assert_eq!(
            sub_models[1].to_old_state_index,
            To1::with_entries(vec![StateIndex::from_raw(0)])
        );
        // State 1 must not be left in the context after building the first sub-model, as its
        // branch would otherwise be mistaken for a branch within the second sub-model.
        assert_eq!(
            sub_models[1].mdp.choice_to_branch,
            Csr::with_entries(vec![BranchIndex::from_raw(0)])
        );
        assert_eq!(
            sub_models[1].choice_exit_values,
            To1::with_entries(vec![0.5 * 0.6 * (1.0 / 0.5)])
        );
    }

    #[test]
    fn collapsed_mec() {
        // States 0 and 1 form a MEC whose only exit is the last choice of state 1.
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s0,
            s1 -> 0.5: s0 & 0.5: s2,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let precomputed_states = S0S1::new(
            To1::with_entries(vec![false, false, false]),
            To1::with_entries(vec![false, false, true]),
        );
        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> =
            Sccs::compute(&model, &precomputed_states);
        let mecs = Mecs::compute(
            &model,
            ExcludeStatesAndChoices::new(
                To1::with_entries(vec![false, false, true]),
                To1::with_entries(vec![false; 4]),
            ),
        );
        let representative = mecs.representative(StateIndex::from_raw(0)).unwrap();
        let values = To1::with_entries(vec![0.0, 0.0, 1.0]);
        let mut context = SubModelConstructionContext::new(&model);

        let mut sub_model: SubModel<_, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> =
            SubModel::empty();
        sub_model.rebuild_from_scc(
            &model,
            sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
            &DominatedByRelation::empty(),
            &mecs,
            &values,
            &(),
            SubModelOrder::BackToFront,
            &mut context,
        );

        // The internal choices are dropped and the exit choice is moved to the representative.
        // Its branch back into the MEC becomes a self-loop, which is removed by rescaling.
        assert_eq!(
            sub_model.to_old_state_index,
            To1::with_entries(vec![representative])
        );
        assert_eq!(
            sub_model.mdp.state_to_choice,
            Csr::with_entries(vec![ChoiceIndex::from_raw(1)])
        );
        assert_eq!(
            sub_model.mdp.choice_to_branch,
            Csr::with_entries(vec![BranchIndex::from_raw(0)])
        );
        assert_eq!(sub_model.choice_exit_values, To1::with_entries(vec![1.0]));
    }

    // TODO: Test submodels with rewards!
}
