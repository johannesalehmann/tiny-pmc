use crate::Model;
use crate::base_model::BaseModel;
use crate::traits::ReadStateSpace;
use typed_index_collections::{Csr, Index, RawIndex, To1};

impl<M: BaseModel, I, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val>
    Model<M, I, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, ()>
{
    pub fn compute_predecessors<PredecessorIdx: Index>(
        self,
    ) -> Model<
        M,
        I,
        ChLabel,
        BrLabel,
        Obs,
        APs,
        Rew,
        Ann,
        Val,
        Predecessors<M::StateIdx, M::ChoiceIdx, M::BranchIdx, PredecessorIdx>,
    > {
        let predecessors = Predecessors::compute(&self.base);
        Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors,
        }
    }

    pub fn with_predecessors<PredecessorIdx: Index>(
        self,
        predecessors: Predecessors<M::StateIdx, M::ChoiceIdx, M::BranchIdx, PredecessorIdx>,
    ) -> Model<
        M,
        I,
        ChLabel,
        BrLabel,
        Obs,
        APs,
        Rew,
        Ann,
        Val,
        Predecessors<M::StateIdx, M::ChoiceIdx, M::BranchIdx, PredecessorIdx>,
    > {
        Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors,
        }
    }
}

impl<M: BaseModel, I, CL, BL, Obs, APs, Rew, Ann, Val, SI: Index, CI: Index, BI: Index, PI: Index>
    Model<M, I, CL, BL, Obs, APs, Rew, Ann, Val, Predecessors<SI, CI, BI, PI>>
{
    pub fn without_predecessors(self) -> Model<M, I, CL, BL, Obs, APs, Rew, Ann, Val, ()> {
        Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: (),
        }
    }
}

pub struct Predecessors<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index, PredecessorIdx: Index>
{
    pub state_to_predecessor: Csr<StateIdx, PredecessorIdx>,
    pub predecessor_to_branch: To1<PredecessorIdx, BranchIdx>,
    pub branch_to_choice: To1<BranchIdx, ChoiceIdx>,
    pub choice_to_state: To1<ChoiceIdx, StateIdx>,
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index, PredecessorIdx: Index>
    Predecessors<StateIdx, ChoiceIdx, BranchIdx, PredecessorIdx>
{
    pub fn compute<
        M: ReadStateSpace<StateIdx = StateIdx, ChoiceIdx = ChoiceIdx, BranchIdx = BranchIdx>,
    >(
        model: &M,
    ) -> Self {
        let mut state_predecessor_count =
            To1::<StateIdx, usize>::with_entries(vec![0; model.states().len()]);

        for state in model.states() {
            for choice in model.choices_of_state(state) {
                for branch in model.branches_of_choice(choice) {
                    let destination = model.branch_destination(branch);
                    state_predecessor_count[destination] += 1;
                }
            }
        }

        let mut state_to_predecessor = Csr::new();
        let mut predecessor_to_branch =
            To1::with_entries(vec![
                BranchIdx::from_raw(BranchIdx::RawType::zero());
                model.branches().len()
            ]);
        let mut branch_to_choice =
            To1::with_entries(vec![
                ChoiceIdx::from_raw(ChoiceIdx::RawType::zero());
                model.branches().len()
            ]);
        let mut choice_to_state =
            To1::with_entries(vec![
                StateIdx::from_raw(StateIdx::RawType::zero());
                model.choices().len()
            ]);

        let mut predecessor_from = PredecessorIdx::from_raw(PredecessorIdx::RawType::zero());
        for (state, predecessor_count) in state_predecessor_count.enumerate() {
            let predecessor_to =
                predecessor_from + PredecessorIdx::RawType::from_usize(*predecessor_count);
            state_to_predecessor.add_entry(state, predecessor_from, predecessor_to);

            predecessor_from = predecessor_to;
        }

        for state in model.states() {
            for choice in model.choices_of_state(state) {
                choice_to_state[choice] = state;
                for branch in model.branches_of_choice(choice) {
                    branch_to_choice[branch] = choice;
                    let destination = model.branch_destination(branch);
                    let predecessor_range = state_to_predecessor.index(destination);
                    if state_predecessor_count[destination] == 0 {
                        panic!(
                            "Predecessor count too low for number of predecessors. This indicates an internal error in the predecessors computation function"
                        )
                    }
                    let index = predecessor_range.index(state_predecessor_count[destination] - 1);
                    predecessor_to_branch[index] = branch;
                    state_predecessor_count[destination] -= 1;
                }
            }
        }

        Predecessors {
            state_to_predecessor,
            predecessor_to_branch,
            branch_to_choice,
            choice_to_state,
        }
    }

    // This function makes tests much easier to write, but should not be used in production.
    #[cfg(test)]
    pub fn nth_predecessor_of_state(&self, state: StateIdx, n: usize) -> BranchIdx {
        self.predecessor_to_branch[self.state_to_predecessor.index(state).index(n)]
    }
}

#[cfg(test)]
mod tests {
    use crate::base_model::Mdp;
    use crate::{Model, PredecessorIndex, StateIndex};
    use typed_index_collections::Index;

    #[test]
    fn self_loop() {
        for padding_states in 0..3usize {
            for choices_per_state in 0..2usize {
                for branches_per_state in 0..2usize {
                    let p = 1.0 / branches_per_state.max(1) as f64;
                    let mut mdp = Mdp::with_default_types();
                    for i in 0..padding_states {
                        let self_index = StateIndex::from_raw(i);
                        mdp.add_state(StateIndex::from_raw(i));

                        for _ in 0..choices_per_state {
                            mdp.add_choice();
                            for _ in 0..branches_per_state {
                                mdp.add_branch(p, self_index);
                            }
                        }
                    }

                    let state = StateIndex::from_raw(padding_states);
                    mdp.add_state(state);
                    let choice = mdp.add_choice();
                    let branch = mdp.add_branch(1.0, state);

                    let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
                    assert_eq!(model.predecessors.choice_to_state[choice], state);
                    assert_eq!(model.predecessors.branch_to_choice[branch], choice);
                    let preds = model.predecessors.state_to_predecessor.index(state);
                    assert_eq!(preds.len(), 1);
                    assert_eq!(
                        model.predecessors.predecessor_to_branch[preds.index(0)],
                        branch
                    );
                }
            }
        }
    }

    #[test]
    fn complex() {
        // Note: We cannot be guaranteed that the predecessors are listed in a specific order, so
        //  this test is actually to specific.
        // If this test breaks, first check whether this is the cause.
        let mut mdp = Mdp::with_default_types();
        let s0 = StateIndex::from_raw(0);
        let s1 = StateIndex::from_raw(1);
        let s2 = StateIndex::from_raw(2);
        let s3 = StateIndex::from_raw(3);
        let s4 = StateIndex::from_raw(4);

        mdp.add_state(s0);

        let c0 = mdp.add_choice();
        let b0 = mdp.add_branch(0.5, s2);
        let b1 = mdp.add_branch(0.5, s4);

        let c1 = mdp.add_choice();
        let b2 = mdp.add_branch(1.0, s2);

        mdp.add_state(s1);
        let c2 = mdp.add_choice();
        let b3 = mdp.add_branch(1.0, s2);

        mdp.add_state(s2);

        mdp.add_state(s3);

        mdp.add_state(s4);
        let c3 = mdp.add_choice();
        let b4 = mdp.add_branch(0.3, s4);
        let b5 = mdp.add_branch(0.7, s1);

        let c4 = mdp.add_choice();
        let b6 = mdp.add_branch(1.0, s4);
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let preds = model.predecessors;

        assert_eq!(preds.state_to_predecessor.index(s0).len(), 0);
        assert_eq!(preds.state_to_predecessor.index(s1).len(), 1);
        assert_eq!(preds.state_to_predecessor.index(s2).len(), 3);
        assert_eq!(preds.state_to_predecessor.index(s3).len(), 0);
        assert_eq!(preds.state_to_predecessor.index(s4).len(), 3);

        assert_eq!(preds.nth_predecessor_of_state(s1, 0), b5);

        assert_eq!(preds.nth_predecessor_of_state(s2, 0), b3);
        assert_eq!(preds.nth_predecessor_of_state(s2, 1), b2);
        assert_eq!(preds.nth_predecessor_of_state(s2, 2), b0);

        assert_eq!(preds.nth_predecessor_of_state(s4, 0), b6);
        assert_eq!(preds.nth_predecessor_of_state(s4, 1), b4);
        assert_eq!(preds.nth_predecessor_of_state(s4, 2), b1);

        assert_eq!(preds.branch_to_choice[b0], c0);
        assert_eq!(preds.branch_to_choice[b1], c0);
        assert_eq!(preds.branch_to_choice[b2], c1);
        assert_eq!(preds.branch_to_choice[b3], c2);
        assert_eq!(preds.branch_to_choice[b4], c3);
        assert_eq!(preds.branch_to_choice[b5], c3);
        assert_eq!(preds.branch_to_choice[b6], c4);

        assert_eq!(preds.choice_to_state[c0], s0);
        assert_eq!(preds.choice_to_state[c1], s0);
        assert_eq!(preds.choice_to_state[c2], s1);
        assert_eq!(preds.choice_to_state[c3], s4);
        assert_eq!(preds.choice_to_state[c4], s4);
    }
}
