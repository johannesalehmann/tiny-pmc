use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use probabilistic_models::typed_index_collections::{Csr, To1, ValuePerIndexSource, index};
use probabilistic_models::{Index, RawIndex};

mod exclusion_criterion;
pub use exclusion_criterion::*;
use typed_index_collections::{IndexRange, IndexRangeIterator};

index!(SccIndex);
index!(SccEntryIndex);
pub struct Sccs<SccIdx: Index, SccEntryIdx: Index, StateIdx: Index> {
    sccs: Csr<SccIdx, SccEntryIdx>,
    scc_entries: To1<SccEntryIdx, StateIdx>,
    is_trivial: To1<SccIdx, bool>,
    state_to_scc: To1<StateIdx, Option<SccIdx>>, // This maps to None for states in s0 or s1
}

impl<ScI: Index, ScEI: Index, SI: Index> Sccs<ScI, ScEI, SI> {
    pub fn compute<M: ReadStateSpace<StateIdx = SI> + ReadPredecessors<StateIdx = SI>>(
        model: &M,
        s0_s1_states: Option<(To1<SI, bool>, To1<SI, bool>)>,
    ) -> Self {
        let mut visited = To1::with_entries(vec![false; model.states().len()]);
        let mut l = Vec::with_capacity(model.states().len());
        let mut scc_entry_count = model.states().len();

        if let Some((s0_states, s1_states)) = &s0_s1_states {
            for state in model.states() {
                if s0_states[state] || s1_states[state] {
                    visited[state] = true;
                    scc_entry_count -= 1;
                }
            }
        }

        for i in model.states() {
            if !visited[i] {
                Self::visit(model, &mut visited, &mut l, i);
            }
        }

        if let Some((s0_states, s1_states)) = &s0_s1_states {
            for state in model.states() {
                visited[state] = s0_states[state] || s1_states[state];
            }
        } else {
            for v in &mut visited {
                *v = false;
            }
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

    /// Appends the states reachable from `state` to `l` in DFS post-order, i.e. every state is
    /// appended only after all states reachable from it have been appended.
    ///
    /// To this end, it maintains a stack of cursors, where each cursor points to the next successor
    /// of a state that needs to be visited. Once all successors are visited, the cursor is popped
    /// from the stack.
    fn visit<M: ReadStateSpace<StateIdx = SI>>(
        model: &M,
        visited: &mut To1<SI, bool>,
        l: &mut Vec<SI>,
        state: SI,
    ) {
        let mut stack = vec![Self::cursor(model, state)];
        visited[state] = true;

        while let Some((i, choices, branches)) = stack.last_mut() {
            let i = *i;

            let descend_into = Self::get_next_unvisited(model, visited, choices, branches);

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

    fn get_next_unvisited<M: ReadStateSpace<StateIdx = SI>>(
        model: &M,
        visited: &mut To1<SI, bool>,
        choices: &mut IndexRangeIterator<<M as ReadStateSpace>::ChoiceIdx>,
        branches: &mut IndexRangeIterator<<M as ReadStateSpace>::BranchIdx>,
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
                *branches = model.branches_of_choice(choice).into_iter();
            } else {
                break;
            }
        }
        descend_into
    }

    /// Creates a DFS stack frame for `state`, with its cursor placed before its first successor.
    fn cursor<M: ReadStateSpace<StateIdx = SI>>(
        model: &M,
        state: SI,
    ) -> (
        SI,
        IndexRangeIterator<M::ChoiceIdx>,
        IndexRangeIterator<M::BranchIdx>,
    ) {
        (
            state,
            model.choices_of_state(state).into_iter(),
            IndexRangeIterator::empty(),
        )
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

    pub fn reverse_topological_ordering(
        &self,
    ) -> ReverseTopologicalOrderIterator<'_, ScI, ScEI, SI> {
        ReverseTopologicalOrderIterator {
            sccs: self,
            current: self.sccs.keys().end(),
        }
    }

    pub fn entries(&self, scc: ScI) -> IndexRange<ScEI> {
        self.sccs.index(scc)
    }

    pub fn state_of_entry(&self, entry: ScEI) -> SI {
        self.scc_entries[entry]
    }

    /// Returns the index of the SCC containing the state, or `None` if the state was excluded
    /// from the SCC computation.
    pub fn scc_index_of_state(&self, state: SI) -> Option<ScI> {
        self.state_to_scc[state]
    }

    pub fn scc(&self, index: ScI) -> Scc<'_, ScI, ScEI, SI> {
        Scc { sccs: self, index }
    }

    pub fn scc_of_state(&self, state: SI) -> Option<Scc<'_, ScI, ScEI, SI>> {
        Some(self.scc(self.scc_index_of_state(state)?))
    }

    pub fn compute_dependencies<SccDependencyIdx: Index, M: ReadStateSpace<StateIdx = SI>>(
        &self,
        model: &M,
    ) -> SccDependencies<ScI, SccDependencyIdx> {
        SccDependencies::compute(model, self)
    }

    pub fn max_size(&self) -> usize {
        self.sccs
            .ranges()
            .into_iter()
            .map(|r| r.len())
            .max()
            .unwrap_or(0)
    }
}

#[derive(Clone, Copy)]
pub struct Scc<'a, ScI: Index, ScEI: Index, SI: Index> {
    sccs: &'a Sccs<ScI, ScEI, SI>,
    index: ScI,
}

impl<'a, ScI: Index, ScEI: Index, SI: Index> Scc<'a, ScI, ScEI, SI> {
    pub fn size(&self) -> usize {
        self.sccs.entries(self.index).len()
    }

    pub fn as_singleton(&self) -> Option<SI> {
        if self.size() == 1 {
            Some(self.states().next().unwrap())
        } else {
            None
        }
    }

    pub fn states(&self) -> impl Iterator<Item = SI> + 'a {
        let sccs = self.sccs;
        let index = self.index;
        sccs.entries(index)
            .into_iter()
            .map(move |entry| sccs.state_of_entry(entry))
    }

    pub fn contains(&self, state: SI) -> bool {
        self.sccs.scc_index_of_state(state) == Some(self.index)
    }
}

pub struct ReverseTopologicalOrderIterator<'a, ScI: Index, ScEI: Index, SI: Index> {
    sccs: &'a Sccs<ScI, ScEI, SI>,
    current: ScI,
}

impl<'a, ScI: Index, ScEI: Index, SI: Index> Iterator
    for ReverseTopologicalOrderIterator<'a, ScI, ScEI, SI>
{
    type Item = Scc<'a, ScI, ScEI, SI>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.raw().as_usize() > 0 {
            self.current -= ScI::RawType::one();
            Some(Scc {
                sccs: self.sccs,
                index: self.current,
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.current.raw().as_usize();
        (size, Some(size))
    }
}

impl<'a, ScI: Index, ScEI: Index, SI: Index> ExactSizeIterator
    for ReverseTopologicalOrderIterator<'a, ScI, ScEI, SI>
{
}

index!(SccDependencyIndex);
pub struct SccDependencies<SccIdx: Index, SccDependencyIdx: Index> {
    scc_dependencies: Csr<SccIdx, SccDependencyIdx>,
    depends_on: To1<SccDependencyIdx, SccIdx>,
}

impl<SccIdx: Index, SccDependencyIdx: Index> SccDependencies<SccIdx, SccDependencyIdx> {
    pub fn compute<M: ReadStateSpace, ScEI: Index>(
        model: &M,
        sccs: &Sccs<SccIdx, ScEI, M::StateIdx>,
    ) -> Self {
        let scc_count = sccs.sccs.keys().len();
        let mut scc_dependencies = Csr::with_capacity(scc_count);
        let mut depends_on = To1::new();

        // For each SCC, remembers the last SCC whose dependency list it was already added to.
        // This lets us deduplicate the (possibly many) edges leading to the same successor SCC
        // in O(1) per edge, without allocating and clearing a fresh set for every SCC.
        let mut last_recorded_for: To1<SccIdx, Option<SccIdx>> =
            To1::with_entries(vec![None; scc_count]);

        for scc in sccs.sccs.keys() {
            for entry in sccs.entries(scc) {
                let state = sccs.state_of_entry(entry);
                for successor in model.successors_of_state(state) {
                    let Some(successor_scc) = sccs.state_to_scc[successor] else {
                        continue;
                    };
                    if successor_scc != scc && last_recorded_for[successor_scc] != Some(scc) {
                        last_recorded_for[successor_scc] = Some(scc);
                        depends_on.add(successor_scc);
                    }
                }
            }
            scc_dependencies.add_entry_unchecked(depends_on.keys().end());
        }

        Self {
            scc_dependencies,
            depends_on,
        }
    }

    pub fn dependencies(&self, scc: SccIdx) -> IndexRange<SccDependencyIdx> {
        self.scc_dependencies.index(scc)
    }

    pub fn dependency_to_scc(&self, dependency: SccDependencyIdx) -> SccIdx {
        self.depends_on[dependency]
    }

    /// Returns the number of SCCs in the longest chain of SCC dependencies.
    pub fn longest_chain(&self) -> usize {
        let mut longest_chain_from: To1<SccIdx, usize> =
            To1::with_entries(vec![0; self.scc_dependencies.keys().len()]);
        let mut longest = 0;

        // As SCCs are already in reverse topological order, a single pass suffices to compute the
        // dependency chain length for every SCC.
        let mut scc = self.scc_dependencies.keys().end();
        while scc.raw().as_usize() > 0 {
            scc -= SccIdx::RawType::one();

            let chain_length = self
                .dependencies(scc)
                .into_iter()
                .map(|dependency| 1 + longest_chain_from[self.dependency_to_scc(dependency)])
                .max()
                .unwrap_or(1);

            longest_chain_from[scc] = chain_length;
            longest = longest.max(chain_length);
        }

        longest
    }
}
#[cfg(test)]
mod tests {
    use super::{SccDependencies, SccDependencyIndex, SccEntryIndex, SccIndex, Sccs};
    use probabilistic_models::mdp;
    use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
    use probabilistic_models::{Model, PredecessorIndex, StateIndex};
    use typed_index_collections::{Csr, Index, To1};

    #[test]
    fn empty() {
        mdp!(mdp = {});
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);
        assert!(sccs.sccs.is_empty());
        assert_eq!(sccs.state_to_scc.len(), 0);
        assert!(sccs.scc_entries.is_empty());
        assert!(sccs.is_trivial.is_empty());
    }

    #[test]
    fn single() {
        mdp!(mdp = { s0 ->, });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);
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
        mdp!(mdp = { s0 -> 1.0: s0 });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);
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
        mdp!(mdp = {
            s0 ->,
            s1 -> 1.0: s1
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);
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
        mdp!(mdp = {
            s0 ->,
            s1 -> 1.0: s2,
            s2 -> 1.0: s2,
            s2 -> 0.5: s2 & 0.5: s1
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);
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

    fn complex_model()
    -> impl ReadStateSpace<StateIdx = StateIndex<usize>> + ReadPredecessors<StateIdx = StateIndex<usize>>
    {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s2,
            s2 -> 1.0: s3,
            s3 -> 1.0: s1,
            s3 -> 1.0: s5,
            s4 -> 0.1: s2 & 0.9: s4,
            s5 -> 0.3: s5 & 0.7: s5
        });

        Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>()
    }

    #[test]
    fn complex() {
        let model = complex_model();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);
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
    fn scc_wrapper() {
        let model = complex_model();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);

        let scc = sccs.scc_of_state(StateIndex::from_raw(1)).unwrap();

        assert_eq!(scc.size(), 3);
        let mut entries: Vec<_> = scc.states().collect();
        entries.sort();
        assert_eq!(
            entries,
            vec![
                StateIndex::from_raw(1),
                StateIndex::from_raw(2),
                StateIndex::from_raw(3)
            ]
        );
        assert!(scc.contains(StateIndex::from_raw(1)));
        assert!(scc.contains(StateIndex::from_raw(3)));
        assert!(!scc.contains(StateIndex::from_raw(0)));
        assert!(!scc.contains(StateIndex::from_raw(4)));
    }

    #[test]
    fn scc_dependencies_complex() {
        let model = complex_model();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);
        let dependencies =
            SccDependencies::<SccIndex<usize>, SccDependencyIndex<usize>>::compute(&model, &sccs);

        let deps_of = |scc: SccIndex<usize>| {
            dependencies
                .dependencies(scc)
                .into_iter()
                .map(|d| dependencies.dependency_to_scc(d))
                .collect::<Vec<_>>()
        };

        assert_eq!(deps_of(SccIndex::from_raw(0)), vec![SccIndex::from_raw(2)]);
        assert_eq!(deps_of(SccIndex::from_raw(1)), vec![SccIndex::from_raw(2)]);
        assert_eq!(deps_of(SccIndex::from_raw(2)), vec![SccIndex::from_raw(3)]);
        assert_eq!(deps_of(SccIndex::from_raw(3)), vec![]);

        // SCC0 -> SCC2 -> SCC3 (and SCC1 -> SCC2 -> SCC3) are the longest chains, each with 3
        // SCCs.
        assert_eq!(dependencies.longest_chain(), 3);
    }

    #[test]
    fn longest_chain_empty() {
        mdp!(mdp = {});
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);
        let dependencies =
            SccDependencies::<SccIndex<usize>, SccDependencyIndex<usize>>::compute(&model, &sccs);

        assert_eq!(dependencies.longest_chain(), 0);
    }

    #[test]
    fn longest_chain_linear() {
        // A chain of four states, each its own trivial SCC: 0 -> 1 -> 2 -> 3.
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s2,
            s2 -> 1.0: s3,
            s3 ->,
        });

        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);
        let dependencies =
            SccDependencies::<SccIndex<usize>, SccDependencyIndex<usize>>::compute(&model, &sccs);

        assert_eq!(dependencies.longest_chain(), 4);
    }

    #[test]
    fn excluded_state_breaks_cycle() {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s2,
            s2 -> 1.0: s0
        });

        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();

        let s0_states = To1::with_entries(vec![false, true, false]);
        let s1_states = To1::with_entries(vec![false, false, false]);
        let sccs = Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(
            &model,
            Some((s0_states, s1_states)),
        );

        assert_eq!(sccs.state_to_scc[StateIndex::from_raw(1)], None);
        let scc0 = sccs.state_to_scc[StateIndex::from_raw(0)].unwrap();
        let scc2 = sccs.state_to_scc[StateIndex::from_raw(2)].unwrap();
        assert_ne!(
            scc0, scc2,
            "excluding state 1 must prevent 0 and 2 from merging into one SCC"
        );
        assert!(sccs.is_trivial[scc0]);
        assert!(sccs.is_trivial[scc2]);

        let dependencies =
            SccDependencies::<SccIndex<usize>, SccDependencyIndex<usize>>::compute(&model, &sccs);
        let deps_of = |scc: SccIndex<usize>| {
            dependencies
                .dependencies(scc)
                .into_iter()
                .map(|d| dependencies.dependency_to_scc(d))
                .collect::<Vec<_>>()
        };

        assert_eq!(deps_of(scc0), vec![]);
        assert_eq!(deps_of(scc2), vec![scc0]);
    }

    #[test]
    fn scc_dependencies_deduplicates_parallel_edges() {
        // State 0 has two choices, one into each state of the 2-state SCC {1, 2}. Both edges
        // land in the same target SCC, so `dependencies` must report it only once rather than
        // once per edge.
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s0 -> 1.0: s2,
            s1 -> 1.0: s2,
            s2 -> 1.0: s1
        });

        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);
        let dependencies =
            SccDependencies::<SccIndex<usize>, SccDependencyIndex<usize>>::compute(&model, &sccs);

        let scc0 = sccs.state_to_scc[StateIndex::from_raw(0)].unwrap();
        let scc1 = sccs.state_to_scc[StateIndex::from_raw(1)].unwrap();
        assert_eq!(sccs.state_to_scc[StateIndex::from_raw(2)], Some(scc1));

        let deps = dependencies
            .dependencies(scc0)
            .into_iter()
            .map(|d| dependencies.dependency_to_scc(d))
            .collect::<Vec<_>>();
        assert_eq!(deps, vec![scc1]);
    }

    #[test]
    fn longest_chain_picks_the_longer_branch() {
        // The dynamic programming of longest chain should correctly choose the largest value of a
        // a state's successors. In the case of state 0, that means it should use the value of state
        // 2 instead of 1.
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s0 -> 1.0: s2,
            s1 ->,
            s2 -> 1.0: s3,
            s3 -> 1.0: s4,
            s4 ->,
        });

        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);
        let dependencies =
            SccDependencies::<SccIndex<usize>, SccDependencyIndex<usize>>::compute(&model, &sccs);

        assert_eq!(dependencies.longest_chain(), 4);
    }

    #[test]
    fn cross_edge_between_siblings_does_not_merge_sccs() {
        // A graph of this form previously caused a too-large SCC to be generated (because the first
        // DFS in the above algorithm did not push nodes in the correct order). This tests for a
        // regression.
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s0 -> 1.0: s2,
            s1 ->,
            s2 -> 1.0: s1,
        });

        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let sccs =
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);

        let scc0 = sccs.scc_index_of_state(StateIndex::from_raw(0)).unwrap();
        let scc1 = sccs.scc_index_of_state(StateIndex::from_raw(1)).unwrap();
        let scc2 = sccs.scc_index_of_state(StateIndex::from_raw(2)).unwrap();

        assert_ne!(scc1, scc2, "states 1 and 2 are not strongly connected");
        assert_ne!(scc0, scc1, "states 0 and 1 are not strongly connected");
        assert_ne!(scc0, scc2, "states 0 and 2 are not strongly connected");
        assert!(sccs.is_trivial[scc0]);
        assert!(sccs.is_trivial[scc1]);
        assert!(sccs.is_trivial[scc2]);

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
            Sccs::<SccIndex<usize>, SccEntryIndex<usize>, StateIndex<usize>>::compute(&model, None);

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

        assert!(sccs.is_trivial[scc0]);
        assert!(!sccs.is_trivial[scc1]);
        assert!(sccs.is_trivial[scc2]);

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
