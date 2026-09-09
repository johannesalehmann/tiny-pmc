use crate::dominated_by::DominatedByRelation;
use crate::sccs::Scc;
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use std::collections::VecDeque;
use typed_index_collections::{Index, RawIndex, To1};

pub struct SubModelConstructionContext<StateIdx: Index> {
    // For the same model, different sub-models may use different new index types (usually the
    // narrowest-possible type). To support all these in a single buffer, we use usize instead of a
    // specific state index type here.
    to_new_state_index: To1<StateIdx, Option<usize>>,
    // Which states were visited in the BFS that is used to determine the order within the sub-model
    visited: To1<StateIdx, bool>,
    visitation_order: Vec<StateIdx>,
    visited_open_list: VecDeque<StateIdx>,
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
        for &state in to_old_state_index {
            self.to_new_state_index[state] = None;
            self.visited[state] = false;
        }
        self.visitation_order.clear();
        self.visited_open_list.clear();
    }
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
        todo!()
    }

    pub fn from_scc<
        M: ReadStateSpace<StateIndex = StateIdx> + ReadPredecessors<StateIdx = StateIdx>,
        ScI: Index,
        ScEI: Index,
    >(
        model: &M,
        scc: Scc<'_, ScI, ScEI, M::StateIndex>,
        dominated_by: &DominatedByRelation<M::StateIndex>,
        values: &To1<M::StateIndex, f64>,
    ) -> Self {
        let mut context = SubModelConstructionContext::new(model);
        let mut sub_model = SubModel::empty();
        sub_model.rebuild_from_scc(model, scc, dominated_by, values, &mut context);
        sub_model
    }

    pub fn rebuild_from_scc<
        M: ReadStateSpace<StateIndex = StateIdx> + ReadPredecessors<StateIdx = StateIdx>,
        ScI: Index,
        ScEI: Index,
    >(
        &mut self,
        model: &M,
        scc: Scc<'_, ScI, ScEI, M::StateIndex>,
        dominated_by: &DominatedByRelation<M::StateIndex>,
        values: &To1<M::StateIndex, f64>,
        context: &mut SubModelConstructionContext<M::StateIndex>,
    ) {
        self.to_old_state_index.clear();
        compute_order(
            model,
            scc,
            dominated_by,
            values,
            context,
            &mut self.to_old_state_index,
        );

        self.mdp.clear();
        self.choice_exit_values.clear();

        for &state in &context.visitation_order {
            let Some(new_state) = context.to_new_state_index[state] else {
                continue; // Skip dominated states
            };
            let new_state = NewSI::from_raw(NewSI::RawType::from_usize(new_state));
            self.mdp.add_state(new_state);
            for choice in model.choices_of_state(state) {
                let choice_index = self.mdp.add_choice();
                let mut to_self = 0.0;
                let mut exit_value = 0.0;
                for branch in model.branches_of_choice(choice) {
                    let mut destination = model.branch_destination(branch);
                    if let Some(dominating_state) = dominated_by.dominated_by(destination) {
                        destination = dominating_state;
                    }
                    let p = model.branch_probability(branch);

                    if destination == state {
                        to_self += p;
                    } else if let Some(target) = context.to_new_state_index[destination] {
                        // We first add the actual probability. After the loop, we then scale the
                        // probability to account for removed self loops.
                        let target = NewSI::from_raw(NewSI::RawType::from_usize(target));
                        self.mdp.add_branch(p, target);
                    } else {
                        exit_value += p * values[destination];
                    }
                }

                let scale_factor = if to_self == 1.0 {
                    0.0
                } else {
                    1.0 / (1.0 - to_self)
                };
                for branch in self.mdp.branches_of_choice(choice_index) {
                    self.mdp.branch_probabilities[branch] *= scale_factor;
                }
                self.choice_exit_values
                    .add_checked(choice_index, exit_value * scale_factor);
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
>(
    model: &M,
    scc: Scc<'_, ScI, ScEI, M::StateIndex>,
    dominated_by: &DominatedByRelation<M::StateIndex>,
    values: &To1<M::StateIndex, f64>,
    context: &mut SubModelConstructionContext<M::StateIndex>,
    to_old_state_index: &mut To1<NewSI, M::StateIndex>,
) {
    // Find states that can leave the SCC into a state with a non-zero value.
    // If an SCC has no such exits, all states within it also have value zero, so the
    // sub-model will be empty.
    // TODO: A BFS is a good starting point, but there are probably algorithms that yield an even
    //  better result (e.g. something inspired by attractor computation or recursive SCC
    //  computation within the SCC)
    for state in scc.states() {
        let mut non_zero_exit = false;
        for successor in model.successors_of_state(state) {
            if !scc.contains(successor) && values[successor] > 0.0 {
                non_zero_exit = true;
                break;
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
        // Dominated states are not added to the sub-model, but they are traversed to visit their
        // predecessors.
        if dominated_by.dominated_by(state).is_none() {
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

#[cfg(test)]
mod tests {
    use super::{SubModel, SubModelConstructionContext};
    use crate::dominated_by::DominatedByRelation;
    use crate::sccs::{SccEntryIndex, SccIndex, Sccs};
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
            Some((
                &To1::with_entries(vec![false, false, false]),
                &To1::with_entries(vec![false, false, true]),
            )),
        );
        let values = To1::with_entries(vec![0.0, 0.0, 1.0]);
        let mut context = SubModelConstructionContext::new(&model);

        let mut sub_model: SubModel<_, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> =
            SubModel::empty();
        sub_model.rebuild_from_scc(
            &model,
            sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
            &DominatedByRelation::empty(),
            &values,
            &mut context,
        );

        assert_eq!(
            sub_model.to_old_state_index,
            To1::with_entries(vec![StateIndex::from_raw(0), StateIndex::from_raw(1)])
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
            To1::with_entries(vec![0.25 * (1.0 / 0.75), 1.0])
        );
        assert_eq!(
            sub_model.choice_exit_values,
            To1::with_entries(vec![0.5 * 1.0 * (1.0 / 0.75), 0.0])
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
            Some((
                &To1::with_entries(vec![false, false, false]),
                &To1::with_entries(vec![false, false, true]),
            )),
        );
        let values = To1::with_entries(vec![0.0, 0.6, 1.0]);
        let mut context = SubModelConstructionContext::new(&model);

        let mut sub_model: SubModel<_, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> =
            SubModel::empty();
        sub_model.rebuild_from_scc(
            &model,
            sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
            &DominatedByRelation::empty(),
            &values,
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
            Some((
                &To1::with_entries(vec![false, false, false]),
                &To1::with_entries(vec![false, false, true]),
            )),
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
            &values,
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
        // Redirecting the branches to state 1 turns them into self loops, which are removed. The
        // first choice thereby becomes a pure self loop, the second one keeps 0.5 self loop.
        assert_eq!(
            sub_model.mdp.choice_to_branch,
            Csr::with_entries(vec![BranchIndex::from_raw(0), BranchIndex::from_raw(0)])
        );
        assert_eq!(
            sub_model.choice_exit_values,
            To1::with_entries(vec![0.0, 0.5 * 1.0 * (1.0 / 0.5)])
        );
    }

    #[test]
    fn the_context_can_be_reused_for_every_scc() {
        // State 0 leaves its own SCC into the SCC of state 1, which is built first. State 1
        // leaves its SCC into the goal state 2, so that it is not pruned from its sub-model.
        mdp!(mdp = {
            s0 -> 0.5: s0 & 0.5: s1,
            s1 -> 0.5: s1 & 0.5: s2,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(
            &model,
            Some((
                &To1::with_entries(vec![false, false, false]),
                &To1::with_entries(vec![false, false, true]),
            )),
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
                &values,
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
}
