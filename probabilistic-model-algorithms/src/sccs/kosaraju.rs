use super::{ExclusionCriterion, Sccs};
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use typed_index_collections::{Csr, Index, IndexRangeIterator, To1, ValuePerIndexSource};

impl<ScI: Index, ScEI: Index, SI: Index> Sccs<ScI, ScEI, SI> {
    pub fn compute_kosaraju<
        M: ReadStateSpace<StateIndex = SI>
            + ReadPredecessors<StateIdx = SI, ChoiceIdx = M::ChoiceIndex, BranchIdx = M::BranchIndex>,
        Ex: ExclusionCriterion<SI, M::ChoiceIndex>,
    >(
        model: &M,
        exclusion_criterion: &Ex,
    ) -> Self {
        let mut visited = To1::with_entries(vec![false; model.states().len()]);
        let mut l = Vec::with_capacity(model.states().len());
        let mut scc_entry_count = model.states().len();

        for state in model.states() {
            if exclusion_criterion.is_state_excluded(state) {
                visited[state] = true;
                scc_entry_count -= 1;
            }
        }

        for i in model.states() {
            if !visited[i] {
                Self::visit(model, exclusion_criterion, &mut visited, &mut l, i);
            }
        }

        for state in model.states() {
            visited[state] = exclusion_criterion.is_state_excluded(state);
        }

        let mut sccs = Csr::new();
        let mut scc_entries = To1::with_capacity(scc_entry_count);
        let mut state_to_scc = To1::with_capacity(model.states().len());

        for &v in l.iter().rev() {
            if !visited[v] {
                visited[v] = true;
                Self::visit_reversed(
                    model,
                    exclusion_criterion,
                    &mut visited,
                    v,
                    &mut scc_entries,
                );
                sccs.add_entry_unchecked(scc_entries.keys().end());
            }
        }

        for _ in model.states() {
            state_to_scc.add(None);
        }

        for (scc, entries) in sccs.ranges().into_iter().enumerate() {
            for entry in entries {
                let state = scc_entries[entry];
                state_to_scc[state] = Some(scc);
            }
        }

        Self {
            sccs,
            scc_entries,
            state_to_scc,
        }
    }

    /// Appends the states reachable from `state` to `l` in DFS post-order, i.e. every state is
    /// appended only after all states reachable from it have been appended.
    ///
    /// To this end, it maintains a stack of cursors, where each cursor points to the next successor
    /// of a state that needs to be visited. Once all successors are visited, the cursor is popped
    /// from the stack.
    fn visit<M: ReadStateSpace<StateIndex = SI>, Ex: ExclusionCriterion<SI, M::ChoiceIndex>>(
        model: &M,
        exclusion_criterion: &Ex,
        visited: &mut To1<SI, bool>,
        l: &mut Vec<SI>,
        state: SI,
    ) {
        let mut stack = vec![Self::cursor(model, state)];
        visited[state] = true;

        while let Some((i, choices, branches)) = stack.last_mut() {
            let i = *i;

            let descend_into =
                Self::get_next_unvisited(model, exclusion_criterion, visited, choices, branches);

            match descend_into {
                Some(destination) => {
                    visited[destination] = true;
                    stack.push(Self::cursor(model, destination));
                }
                None => {
                    l.push(i);
                    stack.pop();
                }
            }
        }
    }

    fn get_next_unvisited<
        M: ReadStateSpace<StateIndex = SI>,
        Ex: ExclusionCriterion<SI, M::ChoiceIndex>,
    >(
        model: &M,
        exclusion_criterion: &Ex,
        visited: &mut To1<SI, bool>,
        choices: &mut IndexRangeIterator<M::ChoiceIndex>,
        branches: &mut IndexRangeIterator<M::BranchIndex>,
    ) -> Option<SI> {
        let mut descend_into = None;
        loop {
            if let Some(branch) = branches.next() {
                let destination = model.branch_destination(branch);
                if !visited[destination] {
                    descend_into = Some(destination);
                    break;
                }
            } else if let Some(choice) = choices.next() {
                if !exclusion_criterion.is_choice_excluded(choice) {
                    *branches = model.branches_of_choice(choice).into_iter();
                }
            } else {
                break;
            }
        }
        descend_into
    }

    /// Creates a DFS stack frame for `state`, with its cursor placed before its first successor.
    fn cursor<M: ReadStateSpace<StateIndex = SI>>(
        model: &M,
        state: SI,
    ) -> (
        SI,
        IndexRangeIterator<M::ChoiceIndex>,
        IndexRangeIterator<M::BranchIndex>,
    ) {
        (
            state,
            model.choices_of_state(state).into_iter(),
            IndexRangeIterator::empty(),
        )
    }

    fn visit_reversed<
        M: ReadPredecessors<StateIdx = SI>,
        Ex: ExclusionCriterion<SI, M::ChoiceIdx>,
    >(
        model: &M,
        exclusion_criterion: &Ex,
        visited: &mut To1<SI, bool>,
        state: SI,
        scc_entries: &mut To1<ScEI, SI>,
    ) -> bool {
        let mut stack = Vec::new();
        stack.push(state);
        let mut is_trivial = true;
        while let Some(state) = stack.pop() {
            scc_entries.add(state);
            for predecessor in model.predecessors_of_state(state) {
                let choice = model.choice_of_predecessor(predecessor);
                if exclusion_criterion.is_choice_excluded(choice) {
                    continue;
                }
                let destination = model.state_of_choice(choice);
                if !visited[destination] {
                    is_trivial = false;
                    visited[destination] = true;
                    stack.push(destination);
                } else {
                    // If an SCC has a single state with the self loop, it is not trivial, but the
                    // above check does not count this edge. Therefore, we handle this separately:
                    if destination == state {
                        is_trivial = false;
                    }
                }
            }
        }
        is_trivial
    }
}

#[cfg(test)]
mod tests {
    use crate::sccs::tests::complex_model;
    use crate::sccs::{ExcludeStatesAndChoices, SccEntryIndex, SccIndex, Sccs};
    use probabilistic_models::mdp;
    use probabilistic_models::{Model, PredecessorIndex, StateIndex};
    use typed_index_collections::{Csr, Index, To1};

    #[test]
    fn empty() {
        mdp!(mdp = {});
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute_kosaraju(
                &model,
                &(),
            );
        assert!(sccs.sccs.is_empty());
        assert_eq!(sccs.state_to_scc.len(), 0);
        assert!(sccs.scc_entries.is_empty());
    }

    #[test]
    fn single() {
        mdp!(mdp = { s0 -> deadlock });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute_kosaraju(
                &model,
                &(),
            );
        assert_eq!(
            sccs.sccs,
            Csr::with_entries(vec![SccEntryIndex::from_raw(1)])
        );
        assert_eq!(
            sccs.state_to_scc,
            To1::with_entries(vec![Some(SccIndex::from_raw(0))])
        );
        assert_eq!(
            sccs.scc_entries,
            To1::with_entries(vec![StateIndex::from_raw(0)])
        );
    }

    #[test]
    fn single_with_loop() {
        mdp!(mdp = { s0 -> 1.0: s0 });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute_kosaraju(
                &model,
                &(),
            );
        assert_eq!(
            sccs.sccs,
            Csr::with_entries(vec![SccEntryIndex::from_raw(1)])
        );
        assert_eq!(
            sccs.state_to_scc,
            To1::with_entries(vec![Some(SccIndex::from_raw(0))])
        );
        assert_eq!(
            sccs.scc_entries,
            To1::with_entries(vec![StateIndex::from_raw(0)])
        );
    }

    #[test]
    fn two_unconnected() {
        mdp!(mdp = {
            s0 -> deadlock,
            s1 -> 1.0: s1
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute_kosaraju(
                &model,
                &(),
            );
        assert_eq!(
            sccs.sccs,
            Csr::with_entries(vec![SccEntryIndex::from_raw(1), SccEntryIndex::from_raw(2)])
        );
        assert_eq!(
            sccs.state_to_scc,
            To1::with_entries(vec![
                Some(SccIndex::from_raw(1)),
                Some(SccIndex::from_raw(0))
            ])
        );
        assert_eq!(
            sccs.scc_entries,
            To1::with_entries(vec![StateIndex::from_raw(1), StateIndex::from_raw(0)])
        );
    }

    #[test]
    fn two_state_loop() {
        mdp!(mdp = {
            s0 -> deadlock,
            s1 -> 1.0: s2,
            s2 -> 1.0: s2,
            s2 -> 0.5: s2 & 0.5: s1
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute_kosaraju(
                &model,
                &(),
            );
        assert_eq!(
            sccs.sccs,
            Csr::with_entries(vec![SccEntryIndex::from_raw(2), SccEntryIndex::from_raw(3)])
        );
        assert_eq!(
            sccs.state_to_scc,
            To1::with_entries(vec![
                Some(SccIndex::from_raw(1)),
                Some(SccIndex::from_raw(0)),
                Some(SccIndex::from_raw(0))
            ])
        );
        assert_eq!(
            sccs.scc_entries,
            To1::with_entries(vec![
                StateIndex::from_raw(1),
                StateIndex::from_raw(2),
                StateIndex::from_raw(0)
            ])
        );
    }

    #[test]
    fn complex() {
        let model = complex_model();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute_kosaraju(
                &model,
                &(),
            );
        assert_eq!(
            sccs.sccs,
            Csr::with_entries(vec![
                SccEntryIndex::from_raw(1),
                SccEntryIndex::from_raw(2),
                SccEntryIndex::from_raw(5),
                SccEntryIndex::from_raw(6)
            ])
        );
        assert_eq!(
            sccs.state_to_scc,
            To1::with_entries(vec![
                Some(SccIndex::from_raw(1)),
                Some(SccIndex::from_raw(2)),
                Some(SccIndex::from_raw(2)),
                Some(SccIndex::from_raw(2)),
                Some(SccIndex::from_raw(0)),
                Some(SccIndex::from_raw(3))
            ])
        );
        assert_eq!(
            sccs.scc_entries,
            To1::with_entries(vec![
                StateIndex::from_raw(4),
                StateIndex::from_raw(0),
                StateIndex::from_raw(1),
                StateIndex::from_raw(3),
                StateIndex::from_raw(2),
                StateIndex::from_raw(5)
            ])
        );

        let order: Vec<Vec<StateIndex<usize>>> = sccs
            .reverse_topological_ordering()
            .map(|scc| {
                let mut states: Vec<_> = scc.states().collect();
                states.sort();
                states
            })
            .collect();
        assert_eq!(
            order,
            vec![
                vec![StateIndex::from_raw(5)],
                vec![
                    StateIndex::from_raw(1),
                    StateIndex::from_raw(2),
                    StateIndex::from_raw(3)
                ],
                vec![StateIndex::from_raw(0)],
                vec![StateIndex::from_raw(4)],
            ]
        )
    }

    #[test]
    fn excluded_choice_breaks_cycle() {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s0,
            s1 -> 1.0: s2,
            s2 -> 1.0: s1
        });

        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let exclusion = ExcludeStatesAndChoices::new(
            To1::with_entries(vec![false, false, false]),
            To1::with_entries(vec![false, true, false, false]),
        );
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute_kosaraju(
                &model, &exclusion,
            );

        let scc0 = sccs.scc_index_of_state(StateIndex::from_raw(0)).unwrap();
        let scc1 = sccs.scc_index_of_state(StateIndex::from_raw(1)).unwrap();
        assert_eq!(sccs.scc_index_of_state(StateIndex::from_raw(2)), Some(scc1));
        assert_ne!(scc0, scc1);
        assert!(
            scc0 < scc1,
            "the edge 0 -> 1 must be respected by the order"
        );
    }

    #[test]
    fn cross_edge_between_siblings_does_not_merge_sccs() {
        // A graph of this form previously caused a too-large SCC to be generated (because the first
        // DFS in the above algorithm did not push nodes in the correct order). This tests for a
        // regression.
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s0 -> 1.0: s2,
            s1 -> deadlock,
            s2 -> 1.0: s1,
        });

        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute_kosaraju(
                &model,
                &(),
            );

        let scc0 = sccs.scc_index_of_state(StateIndex::from_raw(0)).unwrap();
        let scc1 = sccs.scc_index_of_state(StateIndex::from_raw(1)).unwrap();
        let scc2 = sccs.scc_index_of_state(StateIndex::from_raw(2)).unwrap();

        assert_ne!(scc1, scc2, "states 1 and 2 are not strongly connected");
        assert_ne!(scc0, scc1, "states 0 and 1 are not strongly connected");
        assert_ne!(scc0, scc2, "states 0 and 2 are not strongly connected");

        assert!(
            scc0 < scc2,
            "the edge 0 -> 2 must be respected by the order"
        );
        assert!(
            scc2 < scc1,
            "the edge 2 -> 1 must be respected by the order"
        );
    }

    #[test]
    fn cross_edge_does_not_absorb_predecessor_into_scc() {
        // Tests for the same regression as cross_edge_between_siblings_does_not_merge_sccs
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s0 -> 1.0: s2,
            s1 -> 1.0: s3,
            s2 -> 1.0: s1,
            s3 -> 1.0: s1,
        });

        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute_kosaraju(
                &model,
                &(),
            );

        let scc0 = sccs.scc_index_of_state(StateIndex::from_raw(0)).unwrap();
        let scc1 = sccs.scc_index_of_state(StateIndex::from_raw(1)).unwrap();
        let scc2 = sccs.scc_index_of_state(StateIndex::from_raw(2)).unwrap();
        let scc3 = sccs.scc_index_of_state(StateIndex::from_raw(3)).unwrap();

        assert_eq!(scc1, scc3, "states 1 and 3 form an SCC");
        assert_ne!(
            scc2, scc1,
            "state 2 only reaches the SCC {{1, 3}} and cannot be part of it"
        );
        assert_ne!(scc0, scc1);
        assert_ne!(scc0, scc2);

        assert_eq!(
            sccs.entries(scc1)
                .into_iter()
                .map(|entry| sccs.state_of_entry(entry))
                .collect::<std::collections::BTreeSet<_>>(),
            [StateIndex::from_raw(1), StateIndex::from_raw(3)]
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
        );

        assert!(
            scc0 < scc2,
            "the edge 0 -> 2 must be respected by the order"
        );
        assert!(
            scc2 < scc1,
            "the edge 2 -> 1 must be respected by the order"
        );
    }
}
