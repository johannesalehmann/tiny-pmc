use crate::sccs::{ExcludeStatesAndChoices, SccEntryIndex, SccIndex, Sccs};
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use typed_index_collections::{Index, To1};

pub struct Mecs<StateIdx: Index, ChoiceIdx: Index> {
    /// Which state represents the MEC of the given state, if it is in an MEC, otherwise None
    representative: To1<StateIdx, Option<StateIdx>>,
    /// The next member of the MEC (a bit like a linked-list data structure)
    next_member: To1<StateIdx, Option<StateIdx>>,
    /// Whether a given choice is internal to an MEC
    internal_choices: To1<ChoiceIdx, bool>,
}

impl<StateIdx: Index, ChoiceIdx: Index> Mecs<StateIdx, ChoiceIdx> {
    pub fn empty() -> Self {
        Self {
            representative: To1::new(),
            next_member: To1::new(),
            internal_choices: To1::new(),
        }
    }

    pub fn compute<
        M: ReadStateSpace<StateIndex = StateIdx, ChoiceIndex = ChoiceIdx>
            + ReadPredecessors<StateIdx = StateIdx, ChoiceIdx = ChoiceIdx, BranchIdx = M::BranchIndex>,
    >(
        model: &M,
        mut exclusion: ExcludeStatesAndChoices<StateIdx, ChoiceIdx>,
    ) -> Self {
        let mut remaining_choices = To1::with_capacity(model.states().len());
        for state in model.states() {
            let count = model
                .choices_of_state(state)
                .into_iter()
                .filter(|&choice| !exclusion.excluded_choices[choice])
                .count();
            remaining_choices.add_checked(state, count);
        }

        let mut newly_excluded_states = Vec::new();
        loop {
            let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, StateIdx> =
                Sccs::compute(model, &exclusion);
            let mut changed = false;

            for state in model.states() {
                if exclusion.excluded_states[state] {
                    continue;
                }
                let scc = sccs.scc_index_of_state(state);
                for choice in model.choices_of_state(state) {
                    if exclusion.excluded_choices[choice] {
                        continue;
                    }
                    let branches = model.branches_of_choice(choice);
                    let leaves_scc = branches.len() == 0
                        || branches.into_iter().any(|branch| {
                            sccs.scc_index_of_state(model.branch_destination(branch)) != scc
                        });
                    if leaves_scc {
                        exclusion.excluded_choices[choice] = true;
                        remaining_choices[state] -= 1;
                        changed = true;
                    }
                }
                if remaining_choices[state] == 0 {
                    exclusion.excluded_states[state] = true;
                    newly_excluded_states.push(state);
                    changed = true;
                }
            }

            while let Some(state) = newly_excluded_states.pop() {
                for predecessor in model.predecessors_of_state(state) {
                    let choice = model.choice_of_predecessor(predecessor);
                    let source = model.state_of_choice(choice);
                    if exclusion.excluded_states[source] || exclusion.excluded_choices[choice] {
                        continue;
                    }
                    exclusion.excluded_choices[choice] = true;
                    remaining_choices[source] -= 1;
                    if remaining_choices[source] == 0 {
                        exclusion.excluded_states[source] = true;
                        newly_excluded_states.push(source);
                    }
                }
            }

            if !changed {
                return Self::from_sccs(model, &sccs, &exclusion);
            }
        }
    }

    fn from_sccs<M: ReadStateSpace<StateIndex = StateIdx, ChoiceIndex = ChoiceIdx>>(
        model: &M,
        sccs: &Sccs<SccIndex<usize>, SccEntryIndex<usize>, StateIdx>,
        exclusion: &ExcludeStatesAndChoices<StateIdx, ChoiceIdx>,
    ) -> Self {
        let mut representative = To1::with_entries(vec![None; model.states().len()]);
        let mut next_member = To1::with_entries(vec![None; model.states().len()]);
        for scc in sccs.reverse_topological_ordering() {
            let mut states = scc.states();
            let first = states.next().unwrap(); // An SCC cannot be empty
            representative[first] = Some(first);
            let mut previous = first;
            for state in states {
                representative[state] = Some(first);
                next_member[previous] = Some(state);
                previous = state;
            }
        }

        let mut internal_choices = To1::with_entries(vec![false; model.choices().len()]);
        for state in model.states() {
            if representative[state].is_some() {
                for choice in model.choices_of_state(state) {
                    internal_choices[choice] = !exclusion.excluded_choices[choice];
                }
            }
        }

        Self {
            representative,
            next_member,
            internal_choices,
        }
    }

    pub fn representative(&self, state: StateIdx) -> Option<StateIdx> {
        self.representative[state]
    }

    pub fn is_merged_away(&self, state: StateIdx) -> bool {
        self.representative(state)
            .is_some_and(|representative| representative != state)
    }

    // For a representative, yields all states of its MEC. For states outside MECs, yields only the
    // state itself.
    pub fn states_merged_into(&self, state: StateIdx) -> impl Iterator<Item = StateIdx> + '_ {
        std::iter::successors(Some(state), |&member| {
            self.next_member.get(member).copied().flatten()
        })
    }

    pub fn is_internal_choice(&self, choice: ChoiceIdx) -> bool {
        self.internal_choices.get(choice).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::Mecs;
    use crate::sccs::ExcludeStatesAndChoices;
    use probabilistic_models::mdp;
    use probabilistic_models::{ChoiceIndex, Model, PredecessorIndex, StateIndex};
    use typed_index_collections::{Index, To1};

    fn states(indices: &[usize]) -> Vec<StateIndex<usize>> {
        indices.iter().map(|&i| StateIndex::from_raw(i)).collect()
    }

    fn sorted_members(
        mecs: &Mecs<StateIndex<usize>, ChoiceIndex<usize>>,
        state: StateIndex<usize>,
    ) -> Vec<StateIndex<usize>> {
        let representative = mecs.representative(state).unwrap();
        let mut members: Vec<_> = mecs.states_merged_into(representative).collect();
        members.sort();
        members
    }

    #[test]
    fn mec_with_exit() {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s0 -> 1.0: s2,
            s1 -> 1.0: s0,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let mecs = Mecs::compute(
            &model,
            ExcludeStatesAndChoices::new(
                To1::with_entries(vec![false, false, true]),
                To1::with_entries(vec![false; 4]),
            ),
        );

        assert_eq!(sorted_members(&mecs, s0), states(&[0, 1]));
        assert_eq!(mecs.representative(s0), mecs.representative(s1));
        assert_eq!(mecs.representative(s2), None);
        assert_eq!(
            (0..4)
                .map(|c| mecs.is_internal_choice(ChoiceIndex::from_raw(c)))
                .collect::<Vec<_>>(),
            vec![true, false, true, false]
        );
    }

    #[test]
    fn cross_edge_does_not_create_mec() {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s0 -> 1.0: s2,
            s1 -> 1.0: s1,
            s2 -> 1.0: s1
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let mecs = Mecs::compute(
            &model,
            ExcludeStatesAndChoices::new(
                To1::with_entries(vec![false; 3]),
                To1::with_entries(vec![false; 4]),
            ),
        );

        assert_eq!(mecs.representative(s0), None);
        assert_eq!(mecs.representative(s1), Some(s1));
        assert_eq!(mecs.representative(s2), None);
    }

    #[test]
    fn removed_state_splits_scc() {
        // State 2 can only leave, so it is removed, which also removes the choice 1 -> 2.
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s0,
            s1 -> 1.0: s2,
            s2 -> 0.5: s0 & 0.5: s3,
            s3 -> 1.0: s3
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let mecs = Mecs::compute(
            &model,
            ExcludeStatesAndChoices::new(
                To1::with_entries(vec![false, false, false, true]),
                To1::with_entries(vec![false; 5]),
            ),
        );

        assert_eq!(sorted_members(&mecs, s0), states(&[0, 1]));
        assert_eq!(mecs.representative(s2), None);
        assert!(!mecs.is_internal_choice(ChoiceIndex::from_raw(2)));
    }

    #[test]
    fn excluded_choices_prevent_mec() {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s0,
            s1 -> 1.0: s2,
            s2 -> 1.0: s2
        });
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let mecs = Mecs::compute(
            &model,
            ExcludeStatesAndChoices::new(
                To1::with_entries(vec![false, false, true]),
                To1::with_entries(vec![true, false, false, false]),
            ),
        );

        assert_eq!(mecs.representative(s0), None);
        assert_eq!(mecs.representative(s1), None);
    }

    #[test]
    fn empty_mecs() {
        let mecs = Mecs::<StateIndex<usize>, ChoiceIndex<usize>>::empty();
        let state = StateIndex::from_raw(3);
        assert_eq!(mecs.representative(state), None);
        assert!(!mecs.is_merged_away(state));
        assert_eq!(
            mecs.states_merged_into(state).collect::<Vec<_>>(),
            vec![state]
        );
        assert!(!mecs.is_internal_choice(ChoiceIndex::from_raw(0)));
    }
}
