use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use probabilistic_models::typed_index_collections::{Csr, To1, ValuePerIndexSource, index};
use probabilistic_models::{Index, RawIndex};

mod exclusion_criterion;
pub use exclusion_criterion::*;
use typed_index_collections::IndexRange;

index!(SccIndex);
index!(SccEntryIndex);
pub struct Sccs<SccIdx: Index, SccEntryIdx: Index, StateIdx: Index> {
    sccs: Csr<SccIdx, SccEntryIdx>,
    scc_entries: To1<SccEntryIdx, StateIdx>,
    is_trivial: To1<SccIdx, bool>,
    state_to_scc: To1<StateIdx, Option<SccIdx>>, // This maps to None for states excluded by the ExclusionCriterion
}

impl<ScI: Index, ScEI: Index, SI: Index> Sccs<ScI, ScEI, SI> {
    pub fn compute<
        M: ReadStateSpace<StateIdx = SI> + ReadPredecessors<StateIdx = SI>,
        EC: ExclusionCriterion<StateIdx = SI>,
    >(
        model: &M,
        excluded: &EC,
        s0_s1_states: Option<(To1<SI, bool>, To1<SI, bool>)>, // TODO: Perhaps this should instead be handled via the exclusion criterion?
    ) -> Self {
        let mut visited = To1::with_entries(vec![false; model.states().len()]);
        let mut l = Vec::with_capacity(model.states().len());
        let mut scc_entry_count = model.states().len();

        if let Some((s0_states, s1_states)) = s0_s1_states {
            for state in model.states() {
                if s0_states[state] || s1_states[state] {
                    visited[state] = true;
                    scc_entry_count -= 1;
                }
            }
        }

        for excluded in excluded.iter_states() {
            if !visited[excluded] {
                visited[excluded] = true;
                scc_entry_count -= 1;
            }
        }

        for i in model.states() {
            if !visited[i] {
                Self::visit(model, &mut visited, &mut l, i);
            }
        }

        for v in &mut visited {
            *v = false;
        }
        for excluded in excluded.iter_states() {
            visited[excluded] = true;
        }

        let mut sccs = Csr::new();
        let mut scc_entries = To1::with_capacity(scc_entry_count);
        let mut is_trivial = To1::new();
        let mut state_to_scc = To1::with_capacity(model.states().len());

        for &v in l.iter().rev() {
            if !visited[v] {
                visited[v] = true;
                let is_scc_trivial = Self::visit_reversed(model, &mut visited, v, &mut scc_entries);
                sccs.add_entry_unchecked(scc_entries.keys().end());
                is_trivial.add(is_scc_trivial);
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
            is_trivial,
            state_to_scc,
        }
    }

    fn visit<M: ReadStateSpace<StateIdx = SI>>(
        model: &M,
        visited: &mut To1<SI, bool>,
        l: &mut Vec<SI>,
        state: SI,
    ) {
        let mut stack = Vec::new();
        stack.push((state, false));
        visited[state] = true;

        while let Some(top) = stack.pop() {
            match top {
                (i, false) => {
                    stack.push((i, true));
                    for choice in model.choices_of_state(i) {
                        for branch in model.branches_of_choice(choice) {
                            let destination = model.branch_destination(branch);
                            if !visited[destination] {
                                visited[destination] = true;
                                stack.push((destination, false));
                            }
                        }
                    }
                }
                (i, true) => {
                    l.push(i);
                }
            }
        }
    }

    fn visit_reversed<M: ReadPredecessors<StateIdx = SI>>(
        model: &M,
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
                let destination = model.state_of_choice(
                    model.choice_of_branch(model.branch_of_predecessor(predecessor)),
                );
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

    pub fn reverse_topological_ordering(&self) -> impl Iterator<Item = ScI> {
        ReverseTopologicalOrderIterator {
            current: self.sccs.keys().end(),
        }
    }

    // TODO: This interface is not super ergonomic. Once composing CSRs and To1s is possible,
    //  such a composition could be used to directly yield state indices. Currently, this is not
    //  possible without either a custom iterator or allocating
    pub fn entries(&self, scc: ScI) -> IndexRange<ScEI> {
        self.sccs.index(scc)
    }

    pub fn entry_to_state(&self, entry: ScEI) -> SI {
        self.scc_entries[entry]
    }
}

pub struct ReverseTopologicalOrderIterator<ScI: Index> {
    current: ScI,
}

impl<ScI: Index> Iterator for ReverseTopologicalOrderIterator<ScI> {
    type Item = ScI;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.raw().as_usize() > 0 {
            self.current -= ScI::RawType::one();
            Some(self.current)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.current.raw().as_usize();
        (size, Some(size))
    }
}

impl<ScI: Index> ExactSizeIterator for ReverseTopologicalOrderIterator<ScI> {}

#[cfg(test)]
mod tests {
    use super::{NoExclusion, SccEntryIndex, SccIndex, Sccs};
    use probabilistic_models::base_model::Mdp;
    use probabilistic_models::{Model, PredecessorIndex, StateIndex};
    use typed_index_collections::{Csr, Index, To1};

    #[test]
    fn empty() {
        let mdp = Mdp::with_default_types();
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs = Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(
            &model,
            &NoExclusion::new(),
            None,
        );
        assert!(sccs.sccs.is_empty());
        assert_eq!(sccs.state_to_scc.len(), 0);
        assert!(sccs.scc_entries.is_empty());
        assert!(sccs.is_trivial.is_empty());
    }

    #[test]
    fn single() {
        let mut mdp = Mdp::with_default_types();
        mdp.add_state(StateIndex::from_raw(0));
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs = Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(
            &model,
            &NoExclusion::new(),
            None,
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
        assert_eq!(sccs.is_trivial, To1::with_entries(vec![true]));
    }

    #[test]
    fn single_with_loop() {
        let mut mdp = Mdp::with_default_types();
        mdp.add_state(StateIndex::from_raw(0));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(0))]);
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs = Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(
            &model,
            &NoExclusion::new(),
            None,
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
        assert_eq!(sccs.is_trivial, To1::with_entries(vec![false]));
    }

    #[test]
    fn two_unconnected() {
        let mut mdp = Mdp::with_default_types();
        mdp.add_state(StateIndex::from_raw(0));
        mdp.add_state(StateIndex::from_raw(1));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(1))]);
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs = Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(
            &model,
            &NoExclusion::new(),
            None,
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
        assert_eq!(sccs.is_trivial, To1::with_entries(vec![false, true]));
    }

    #[test]
    fn two_state_loop() {
        let mut mdp = Mdp::with_default_types();
        mdp.add_state(StateIndex::from_raw(0));
        mdp.add_state(StateIndex::from_raw(1));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(2))]);
        mdp.add_state(StateIndex::from_raw(2));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(2))]);
        mdp.add_choice_from_slice(&[
            (0.5, StateIndex::from_raw(2)),
            (0.5, StateIndex::from_raw(1)),
        ]);
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs = Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(
            &model,
            &NoExclusion::new(),
            None,
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
        assert_eq!(sccs.is_trivial, To1::with_entries(vec![false, true]));
    }

    #[test]
    fn complex() {
        let mut mdp = Mdp::with_default_types();
        mdp.add_state(StateIndex::from_raw(0));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(1))]);

        mdp.add_state(StateIndex::from_raw(1));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(2))]);

        mdp.add_state(StateIndex::from_raw(2));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(3))]);

        mdp.add_state(StateIndex::from_raw(3));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(1))]);
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(5))]);

        mdp.add_state(StateIndex::from_raw(4));
        mdp.add_choice_from_slice(&[
            (0.1, StateIndex::from_raw(2)),
            (0.9, StateIndex::from_raw(4)),
        ]);

        mdp.add_state(StateIndex::from_raw(5));
        mdp.add_choice_from_slice(&[
            (0.3, StateIndex::from_raw(5)),
            (0.7, StateIndex::from_raw(5)),
        ]);

        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs = Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(
            &model,
            &NoExclusion::new(),
            None,
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
        assert_eq!(
            sccs.is_trivial,
            To1::with_entries(vec![false, true, false, false])
        );

        assert_eq!(
            sccs.reverse_topological_ordering().collect::<Vec<_>>(),
            vec![
                SccIndex::from_raw(3),
                SccIndex::from_raw(2),
                SccIndex::from_raw(1),
                SccIndex::from_raw(0),
            ]
        )
    }
}
