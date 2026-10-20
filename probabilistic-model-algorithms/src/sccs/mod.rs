use probabilistic_models::traits::ReadStateSpace;
use probabilistic_models::typed_index_collections::{Csr, To1, ValuePerIndexSource};
use probabilistic_models::{Index, Ix, RawIndex, StateIndex};

mod exclusion_criterion;
pub use exclusion_criterion::*;
mod kosaraju;
mod tarjan;
pub use tarjan::SccChainWeight;
use typed_index_collections::IndexRange;

typed_index_collections::index!(SccIndex<I: Ix> => I::State);
typed_index_collections::index!(SccEntryIndex<I: Ix> => I::State);
pub struct Sccs<I: Ix> {
    sccs: Csr<SccIndex<I>, SccEntryIndex<I>>,
    scc_entries: To1<SccEntryIndex<I>, StateIndex<I>>,
    state_to_scc: To1<StateIndex<I>, Option<SccIndex<I>>>, // This maps to None for excluded states
}

impl<I: Ix> Sccs<I> {
    pub fn new<Inner: IntoIterator<Item = StateIndex<I>>, Outer: IntoIterator<Item = Inner>>(
        scc_iter: Outer,
        state_count: usize,
    ) -> Self {
        let mut sccs = Csr::new();
        let mut scc_entries = To1::new();
        let mut state_to_scc = To1::with_entries(vec![None; state_count]);
        for scc in scc_iter {
            let scc_index = sccs.add_empty_entry();
            for state in scc {
                let entry_index = scc_entries.add(state);
                sccs.extend_last_entry(entry_index + I::State::one());
                state_to_scc[state] = Some(scc_index);
            }
        }
        Self {
            sccs,
            scc_entries,
            state_to_scc,
        }
    }

    pub fn reverse_topological_ordering(&self) -> ReverseTopologicalOrderIterator<'_, I> {
        ReverseTopologicalOrderIterator {
            sccs: self,
            current: self.sccs.keys().end(),
        }
    }

    pub fn entries(&self, scc: SccIndex<I>) -> IndexRange<SccEntryIndex<I>> {
        self.sccs.index(scc)
    }

    pub fn state_of_entry(&self, entry: SccEntryIndex<I>) -> StateIndex<I> {
        self.scc_entries[entry]
    }

    /// Returns the index of the SCC containing the state, or `None` if the state was excluded
    /// from the SCC computation.
    pub fn scc_index_of_state(&self, state: StateIndex<I>) -> Option<SccIndex<I>> {
        self.state_to_scc[state]
    }

    pub fn scc(&self, index: SccIndex<I>) -> Scc<'_, I> {
        Scc { sccs: self, index }
    }

    pub fn scc_of_state(&self, state: StateIndex<I>) -> Option<Scc<'_, I>> {
        Some(self.scc(self.scc_index_of_state(state)?))
    }

    pub fn compute_dependencies<M: ReadStateSpace<Ix = I>, Ex: ExclusionCriterion<I>>(
        &self,
        model: &M,
        exclusion_criterion: &Ex,
    ) -> SccDependencies<I> {
        SccDependencies::compute(model, self, exclusion_criterion)
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
pub struct Scc<'a, I: Ix> {
    sccs: &'a Sccs<I>,
    index: SccIndex<I>,
}

impl<'a, I: Ix> Scc<'a, I> {
    pub fn size(&self) -> usize {
        self.sccs.entries(self.index).len()
    }

    pub fn as_singleton(&self) -> Option<StateIndex<I>> {
        if self.size() == 1 {
            Some(self.states().next().unwrap())
        } else {
            None
        }
    }

    pub fn states(&self) -> impl Iterator<Item = StateIndex<I>> + 'a {
        let sccs = self.sccs;
        let index = self.index;
        sccs.entries(index)
            .into_iter()
            .map(move |entry| sccs.state_of_entry(entry))
    }

    pub fn contains(&self, state: StateIndex<I>) -> bool {
        self.sccs.scc_index_of_state(state) == Some(self.index)
    }

    pub fn get_index(&self) -> SccIndex<I> {
        self.index
    }
}

pub struct ReverseTopologicalOrderIterator<'a, I: Ix> {
    sccs: &'a Sccs<I>,
    current: SccIndex<I>,
}

impl<'a, I: Ix> Iterator for ReverseTopologicalOrderIterator<'a, I> {
    type Item = Scc<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.raw().as_usize() > 0 {
            self.current -= I::State::one();
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

impl<'a, I: Ix> ExactSizeIterator for ReverseTopologicalOrderIterator<'a, I> {}

typed_index_collections::index!(SccDependencyIndex<I: Ix> => I::Branch);
pub struct SccDependencies<I: Ix> {
    scc_dependencies: Csr<SccIndex<I>, SccDependencyIndex<I>>,
    depends_on: To1<SccDependencyIndex<I>, SccIndex<I>>,
}

impl<I: Ix> SccDependencies<I> {
    pub fn compute<M: ReadStateSpace<Ix = I>, Ex: ExclusionCriterion<I>>(
        model: &M,
        sccs: &Sccs<I>,
        exclusion_criterion: &Ex,
    ) -> Self {
        let scc_count = sccs.sccs.keys().len();
        let mut scc_dependencies = Csr::with_capacity(scc_count);
        let mut depends_on = To1::new();

        // For each SCC, remembers the last SCC whose dependency list it was already added to.
        // This lets us deduplicate the (possibly many) edges leading to the same successor SCC
        // in O(1) per edge, without allocating and clearing a fresh set for every SCC.
        let mut last_recorded_for: To1<SccIndex<I>, Option<SccIndex<I>>> =
            To1::with_entries(vec![None; scc_count]);

        for scc in sccs.sccs.keys() {
            for entry in sccs.entries(scc) {
                let state = sccs.state_of_entry(entry);
                for choice in model.choices_of_state(state) {
                    if exclusion_criterion.is_choice_excluded(choice) {
                        continue;
                    }
                    for branch in model.branches_of_choice(choice) {
                        let successor = model.branch_destination(branch);
                        let Some(successor_scc) = sccs.state_to_scc[successor] else {
                            continue;
                        };
                        if successor_scc != scc && last_recorded_for[successor_scc] != Some(scc) {
                            last_recorded_for[successor_scc] = Some(scc);
                            depends_on.add(successor_scc);
                        }
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

    pub fn dependencies(&self, scc: SccIndex<I>) -> IndexRange<SccDependencyIndex<I>> {
        self.scc_dependencies.index(scc)
    }

    pub fn dependency_to_scc(&self, dependency: SccDependencyIndex<I>) -> SccIndex<I> {
        self.depends_on[dependency]
    }

    /// Returns the number of SCCs in the longest chain of SCC dependencies.
    pub fn longest_chain(&self) -> usize {
        self.heaviest_chain(|_| 1)
    }

    /// Returns the number of SCCs with more than one state in the chain of SCC dependencies that
    /// contains the most such SCCs.
    pub fn longest_non_singleton_chain(&self, sccs: &Sccs<I>) -> usize {
        self.heaviest_non_singleton_chain(sccs, |_| 1)
    }

    /// Like [`Self::heaviest_chain`], but SCCs with only one state have weight 0.
    pub fn heaviest_non_singleton_chain(
        &self,
        sccs: &Sccs<I>,
        weight: impl Fn(SccIndex<I>) -> usize,
    ) -> usize {
        self.heaviest_chain(|scc| {
            if sccs.scc(scc).as_singleton().is_some() {
                0
            } else {
                weight(scc)
            }
        })
    }

    /// Returns the largest total weight of any chain of SCC dependencies.
    pub fn heaviest_chain(&self, weight: impl Fn(SccIndex<I>) -> usize) -> usize {
        let mut longest_chain_from: To1<SccIndex<I>, usize> =
            To1::with_entries(vec![0; self.scc_dependencies.keys().len()]);
        let mut longest = 0;

        // As SCCs are already in reverse topological order, a single pass suffices to compute the
        // dependency chain length for every SCC.
        let mut scc = self.scc_dependencies.keys().end();
        while scc.raw().as_usize() > 0 {
            scc -= I::State::one();

            let chain_length = self
                .dependencies(scc)
                .into_iter()
                .map(|dependency| longest_chain_from[self.dependency_to_scc(dependency)])
                .max()
                .unwrap_or(0)
                + weight(scc);

            longest_chain_from[scc] = chain_length;
            longest = longest.max(chain_length);
        }

        longest
    }
}
#[cfg(test)]
mod tests {
    // TODO: Tests currently only cover kosajaru, even though tarjan is used more often given its
    //  better performance.
    use super::{ExcludeStatesAndChoices, SccDependencies, SccIndex, Sccs};
    use crate::value_iteration::precomputed_states::S0S1;
    use probabilistic_models::mdp;
    use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
    use probabilistic_models::{Model, StateIndex, Uniform};
    use typed_index_collections::{Index, To1};

    pub(super) fn complex_model()
    -> impl ReadStateSpace<Ix = Uniform<usize>> + ReadPredecessors<Ix = Uniform<usize>> {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s2,
            s2 -> 1.0: s3,
            s3 -> 1.0: s1,
            s3 -> 1.0: s5,
            s4 -> 0.1: s2 & 0.9: s4,
            s5 -> 0.3: s5 & 0.7: s5
        });

        Model::new(mdp).compute_predecessors()
    }

    #[test]
    fn scc_wrapper() {
        let model = complex_model();
        let sccs = Sccs::<Uniform<usize>>::compute_kosaraju(&model, &());

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
        let sccs = Sccs::<Uniform<usize>>::compute_kosaraju(&model, &());
        let dependencies = SccDependencies::<Uniform<usize>>::compute(&model, &sccs, &());

        let deps_of = |scc: SccIndex<Uniform<usize>>| {
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
        let model = Model::new(mdp).compute_predecessors();
        let sccs = Sccs::<Uniform<usize>>::compute_kosaraju(&model, &());
        let dependencies = SccDependencies::<Uniform<usize>>::compute(&model, &sccs, &());

        assert_eq!(dependencies.longest_chain(), 0);
    }

    #[test]
    fn longest_chain_linear() {
        // A chain of four states, each its own trivial SCC: 0 -> 1 -> 2 -> 3.
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s2,
            s2 -> 1.0: s3,
            s3 -> deadlock
        });

        let model = Model::new(mdp).compute_predecessors();
        let sccs = Sccs::<Uniform<usize>>::compute_kosaraju(&model, &());
        let dependencies = SccDependencies::<Uniform<usize>>::compute(&model, &sccs, &());

        assert_eq!(dependencies.longest_chain(), 4);
    }

    #[test]
    fn excluded_state_breaks_cycle() {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s2,
            s2 -> 1.0: s0
        });

        let model = Model::new(mdp).compute_predecessors();

        let precomputed_states = S0S1::new(
            To1::with_entries(vec![false, true, false]),
            To1::with_entries(vec![false, false, false]),
        );
        let sccs = Sccs::<Uniform<usize>>::compute_kosaraju(&model, &precomputed_states);

        assert_eq!(sccs.state_to_scc[StateIndex::from_raw(1)], None);
        let scc0 = sccs.state_to_scc[StateIndex::from_raw(0)].unwrap();
        let scc2 = sccs.state_to_scc[StateIndex::from_raw(2)].unwrap();
        assert_ne!(
            scc0, scc2,
            "excluding state 1 must prevent 0 and 2 from merging into one SCC"
        );

        let dependencies =
            SccDependencies::<Uniform<usize>>::compute(&model, &sccs, &precomputed_states);
        let deps_of = |scc: SccIndex<Uniform<usize>>| {
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
    fn scc_dependencies_ignore_excluded_choices() {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s0 -> 1.0: s2,
            s1 -> deadlock,
            s2 -> deadlock
        });

        let model = Model::new(mdp).compute_predecessors();
        let exclusion = ExcludeStatesAndChoices::new(
            To1::with_entries(vec![false, false, false]),
            To1::with_entries(vec![true, false]),
        );
        let sccs = Sccs::<Uniform<usize>>::compute_kosaraju(&model, &exclusion);
        let dependencies = SccDependencies::<Uniform<usize>>::compute(&model, &sccs, &exclusion);

        let scc0 = sccs.scc_index_of_state(StateIndex::from_raw(0)).unwrap();
        let scc2 = sccs.scc_index_of_state(StateIndex::from_raw(2)).unwrap();
        let deps = dependencies
            .dependencies(scc0)
            .into_iter()
            .map(|d| dependencies.dependency_to_scc(d))
            .collect::<Vec<_>>();
        assert_eq!(deps, vec![scc2]);
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

        let model = Model::new(mdp).compute_predecessors();
        let sccs = Sccs::<Uniform<usize>>::compute_kosaraju(&model, &());
        let dependencies = SccDependencies::<Uniform<usize>>::compute(&model, &sccs, &());

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
            s1 -> deadlock,
            s2 -> 1.0: s3,
            s3 -> 1.0: s4,
            s4 -> deadlock,
        });

        let model = Model::new(mdp).compute_predecessors();
        let sccs = Sccs::<Uniform<usize>>::compute_kosaraju(&model, &());
        let dependencies = SccDependencies::<Uniform<usize>>::compute(&model, &sccs, &());

        assert_eq!(dependencies.longest_chain(), 4);
    }
}
