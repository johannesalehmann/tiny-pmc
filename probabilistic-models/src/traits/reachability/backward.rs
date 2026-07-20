use crate::traits::StateSet;
use crate::traits::predecessors::ReadPredecessors;
use typed_index_collections::{Index, To1};

pub trait BackwardReachability {
    type StateIdx: Index;
    fn backward_reachable_states<S: StateSet<Self::StateIdx>>(
        &self,
        from: S,
    ) -> To1<Self::StateIdx, bool>;
}

impl<M: ReadPredecessors> BackwardReachability for M {
    type StateIdx = M::StateIdx;
    fn backward_reachable_states<S: StateSet<Self::StateIdx>>(
        &self,
        from: S,
    ) -> To1<Self::StateIdx, bool> {
        let mut open_states = from.iter().collect::<Vec<_>>();
        let mut buffer = To1::with_entries(vec![false; self.predecessor_states().len()]);

        for &state in &open_states {
            buffer[state] = true;
        }

        while let Some(open) = open_states.pop() {
            for predecessor in self.predecessors_of_state(open) {
                let branch = self.branch_of_predecessor(predecessor);
                let choice = self.choice_of_branch(branch);
                let source = self.state_of_choice(choice);
                if !buffer[source] {
                    buffer[source] = true;
                    open_states.push(source);
                }
            }
        }

        buffer
    }
}

#[cfg(test)]
mod tests {
    use crate::base_model::Mdp;
    use crate::traits::BackwardReachability;
    use crate::{Model, PredecessorIndex, StateIndex};
    use typed_index_collections::Index;

    #[test]
    fn chain() {
        for (self_loop, extra_jump) in [(false, false), (false, true), (true, false), (true, true)]
        {
            for states in 2..5 {
                let mut mdp = Mdp::with_default_types();
                for i in 0..states {
                    mdp.add_state(StateIndex::from_raw(i));
                    if i + 1 < states && (!extra_jump || i + 2 >= states) {
                        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(i + 1))]);
                    }
                    if i + 2 < states && extra_jump {
                        mdp.add_choice_from_slice(&[
                            (0.5, StateIndex::from_raw(i + 1)),
                            (0.5, StateIndex::from_raw(i + 2)),
                        ]);
                    }
                    if self_loop {
                        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(i))]);
                    }
                }
                let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

                for check in 0..states {
                    let reachable = model.backward_reachable_states(StateIndex::from_raw(check));
                    for i in 0..states {
                        assert_eq!(
                            reachable[StateIndex::from_raw(i)],
                            i <= check,
                            "Expected state {i} to be{} reachable from states {check}, but it was{} (total states: {states}, self loop: {self_loop}, extra_jump: {extra_jump})",
                            if i <= check { "" } else { " not" },
                            if reachable[StateIndex::from_raw(i)] {
                                ""
                            } else {
                                " not"
                            }
                        );
                    }
                }
            }
        }
    }
}
