use crate::dominated_by::DominatedByRelation;
use crate::sccs::Sccs;
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::ReadStateSpace;
use typed_index_collections::{Index, To1};

pub struct SubModelContext<StateIdx: Index, NewSI: Index> {
    to_new_state_index: To1<StateIdx, Option<NewSI>>,
}

impl<StateIdx: Index, NewSI: Index> SubModelContext<StateIdx, NewSI> {
    pub fn new<M: ReadStateSpace<StateIdx = StateIdx>>(model: &M) -> Self {
        Self {
            to_new_state_index: To1::with_entries(vec![None; model.states().len()]),
        }
    }
}

pub struct SubModel<StateIdx: Index, NewSI: Index, NewCI: Index, NewBI: Index> {
    pub mdp: Mdp<NewSI, NewCI, NewBI>,
    /// The value that external states (i.e. those outside the SCC) contribute to a choice.
    pub choice_exit_values: To1<NewCI, f64>,
    pub to_old_state_index: To1<NewSI, StateIdx>,
}

// TODO: As we know the size of the SCC before, we could determine the smallest possible index types
//  for each SCC and build the sub-model using those.
pub fn build_sub_model<
    M: ReadStateSpace,
    ScI: Index,
    ScEI: Index,
    NewSI: Index,
    NewCI: Index,
    NewBI: Index,
>(
    model: &M,
    scc: ScI,
    sccs: &Sccs<ScI, ScEI, M::StateIdx>,
    dominated_by: &DominatedByRelation<M::StateIdx>,
    values: &To1<M::StateIdx, f64>,
    context: &mut SubModelContext<M::StateIdx, NewSI>,
) -> SubModel<M::StateIdx, NewSI, NewCI, NewBI> {
    let to_new_state_index = &mut context.to_new_state_index;
    let mut to_old_state_index: To1<NewSI, M::StateIdx> =
        To1::with_capacity(sccs.entries(scc).len());
    for scc_entry in sccs.entries(scc) {
        let state = sccs.state_of_entry(scc_entry);
        if dominated_by.dominated_by(state).is_none() {
            let new_index = to_old_state_index.add(state);
            to_new_state_index[state] = Some(new_index);
        }
    }

    let mut mdp = Mdp::default();
    let mut choice_exit_values = To1::new();

    for scc_entry in sccs.entries(scc) {
        let state = sccs.state_of_entry(scc_entry);
        // Dominated states are not part of the sub-model, as they are represented by the state
        // dominating them.
        let Some(new_state) = to_new_state_index[state] else {
            continue;
        };
        mdp.add_state(new_state);
        for choice in model.choices_of_state(state) {
            let choice_index = mdp.add_choice();
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
                } else if let Some(target) = to_new_state_index[destination] {
                    // We first add the true probability. After the loop, we then scale the
                    // probability to account for removed self loops.
                    mdp.add_branch(p, target);
                } else {
                    exit_value += p * values[destination];
                }
            }

            let scale_factor = if to_self == 1.0 {
                // If the model is well-formed, then the choice has no other branches and thus
                // cannot contribute any value. Scaling with zero avoids dividing by zero.
                0.0
            } else {
                1.0 / (1.0 - to_self)
            };
            for branch in mdp.branches_of_choice(choice_index) {
                mdp.branch_probabilities[branch] *= scale_factor;
            }
            choice_exit_values.add_checked(choice_index, exit_value * scale_factor);
        }
    }

    for &state in &to_old_state_index {
        to_new_state_index[state] = None;
    }

    SubModel {
        mdp,
        choice_exit_values,
        to_old_state_index,
    }
}

#[cfg(test)]
mod tests {
    use super::{SubModel, SubModelContext, build_sub_model};
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
                To1::with_entries(vec![false, false, false]),
                To1::with_entries(vec![false, false, true]),
            )),
        );
        let values = To1::with_entries(vec![0.0, 0.0, 1.0]);
        let mut context = SubModelContext::new(&model);

        let sub_model =
            build_sub_model::<_, _, _, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>(
                &model,
                sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
                &sccs,
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
                To1::with_entries(vec![false, false, false]),
                To1::with_entries(vec![false, false, true]),
            )),
        );
        let values = To1::with_entries(vec![0.0, 0.6, 1.0]);
        let mut context = SubModelContext::new(&model);

        let sub_model =
            build_sub_model::<_, _, _, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>(
                &model,
                sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
                &sccs,
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
                To1::with_entries(vec![false, false, false]),
                To1::with_entries(vec![false, false, true]),
            )),
        );
        let values = To1::with_entries(vec![0.0, 0.0, 1.0]);
        let dominated_by = DominatedByRelation::with_entries(To1::with_entries(vec![
            None,
            Some(StateIndex::from_raw(0)),
            None,
        ]));
        let mut context = SubModelContext::new(&model);

        let sub_model =
            build_sub_model::<_, _, _, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>(
                &model,
                sccs.scc_of_state(StateIndex::from_raw(0)).unwrap(),
                &sccs,
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
        // State 0 leaves its own SCC into the SCC of state 1, which is built first.
        mdp!(mdp = {
            s0 -> 0.5: s0 & 0.5: s1,
            s1 -> 1.0: s1
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(&model, None);
        let values = To1::with_entries(vec![0.0, 0.6]);
        let mut context = SubModelContext::new(&model);

        let mut sub_models: Vec<
            SubModel<StateIndex<usize>, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>,
        > = Vec::new();
        for scc in sccs.reverse_topological_ordering() {
            sub_models.push(build_sub_model(
                &model,
                scc,
                &sccs,
                &DominatedByRelation::empty(),
                &values,
                &mut context,
            ));
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
