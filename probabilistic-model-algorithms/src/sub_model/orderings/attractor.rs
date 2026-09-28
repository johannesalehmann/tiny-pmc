use crate::mecs::Mecs;
use crate::sccs::Scc;
use crate::sub_model::{RewardsSource, StateOrdering};
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use typed_index_collections::{Index, To1};

// The attractor algorithm computes which fraction of probability mass is known for each *choice*.
// This enum specifies how to use the choice values to state values.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AttractorChoiceMode {
    BestChoice,
    WorstChoice,
    AverageChoice,
}

pub struct Attractor {
    choice_mode: AttractorChoiceMode,
}

impl Attractor {
    pub fn new(choice_mode: AttractorChoiceMode) -> Self {
        Self { choice_mode }
    }
}

impl StateOrdering for Attractor {
    type Context<SI: Index, CI: Index> = AttractorContext<SI, CI>;

    fn create_context<M: ReadStateSpace>(
        &self,
        model: &M,
    ) -> Self::Context<M::StateIndex, M::ChoiceIndex> {
        AttractorContext {
            queue: BinaryHeap::new(),
            known_mass: To1::with_entries(vec![0.0; model.choices().len()]),
            non_self_loop_mass: To1::with_entries(vec![0.0; model.choices().len()]),
            visited: To1::with_entries(vec![false; model.states().len()]),
        }
    }

    fn compute_ordering<
        M: ReadStateSpace<StateIndex = SI, ChoiceIndex = CI, BranchIndex = BI>
            + ReadPredecessors<StateIdx = SI, ChoiceIdx = CI, BranchIdx = BI>,
        SI: Index,
        CI: Index,
        BI: Index,
        ScI: Index,
        ScEI: Index,
        NewSI: Index,
        Rew: RewardsSource<SI, CI>,
    >(
        &self,
        model: &M,
        scc: Scc<'_, ScI, ScEI, SI>,
        mecs: &Mecs<SI, CI>,
        _values: &To1<SI, f64>,
        _rewards: &Rew,
        context: &mut Self::Context<SI, CI>,
        to_old_state_index: &mut To1<NewSI, SI>,
    ) {
        for state in scc.states() {
            for choice in model.choices_of_state(state) {
                let mut known = 0.0;
                let mut non_self_loops = 0.0;
                for branch in model.branches_of_choice(choice) {
                    let dest = model.branch_destination(branch);
                    if dest != state
                        && !mecs
                            .representative(dest)
                            .map(|r| Some(r) == mecs.representative(state))
                            .unwrap_or(false)
                    {
                        non_self_loops += model.branch_probability(branch);
                        if !scc.contains(dest) {
                            known += model.branch_probability(branch);
                        }
                    }
                }
                context.known_mass[choice] = known;
                context.non_self_loop_mass[choice] = non_self_loops;
            }
        }
        // The priority of an MEC depends on the choices of all its members, so this requires a
        // separate pass
        for state in scc.states() {
            if !mecs.is_merged_away(state) {
                context.queue.push((
                    context.state_priority(
                        mecs.states_merged_into(state)
                            .map(|s| model.choices_of_state(s))
                            .flatten(),
                        self.choice_mode,
                    ),
                    state,
                ));
            }
        }

        while let Some((_, state)) = context.queue.pop() {
            if context.visited[state] {
                // There may be duplicates in the queue (see below)
                continue;
            }
            context.visited[state] = true;
            to_old_state_index.add(state);
            for predecessor in mecs
                .states_merged_into(state)
                .map(|s| model.predecessors_of_state(s).into_iter())
                .flatten()
            {
                let branch = model.branch_of_predecessor(predecessor);
                let choice = model.choice_of_branch(branch);
                let pred_state = model.state_of_choice(choice);
                let pred_state = mecs.representative(pred_state).unwrap_or(pred_state);
                if !scc.contains(pred_state) {
                    continue;
                }
                context.known_mass[choice] += model.branch_probability(branch);
                let new_priority = context.state_priority(
                    mecs.states_merged_into(pred_state)
                        .map(|s| model.choices_of_state(s))
                        .flatten(),
                    self.choice_mode,
                );

                // This creates a duplicate, but the new entry will be before the old entries
                context.queue.push((new_priority, pred_state));
            }
        }

        // Reset context:
        // TODO: This might not actually be necessary, as SCCs (from which these sub-models are
        //  probably built), are disjoint. However, whether this is required is never explicitly
        //  stated.
        context.queue.clear();
        for state in scc.states() {
            for choice in model.choices_of_state(state) {
                context.known_mass[choice] = 0.0;
                context.non_self_loop_mass[choice] = 0.0;
            }
            context.visited[state] = false;
        }
    }
}

struct Priority {
    value: f64,
}

impl Priority {
    pub fn new(value: f64) -> Self {
        assert!(value.is_finite());
        Self { value }
    }
}

impl PartialEq for Priority {
    fn eq(&self, other: &Self) -> bool {
        self.value.total_cmp(&other.value).is_eq()
    }
}

impl Eq for Priority {}
impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.value.total_cmp(&other.value))
    }
}
impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.total_cmp(&other.value)
    }
}

pub struct AttractorContext<SI: Index, CI: Index> {
    queue: BinaryHeap<(Priority, SI)>,
    known_mass: To1<CI, f64>,
    non_self_loop_mass: To1<CI, f64>,
    visited: To1<SI, bool>,
}

impl<SI: Index, CI: Index> AttractorContext<SI, CI> {
    fn state_priority(
        &self,
        choices: impl Iterator<Item = CI>,
        mode: AttractorChoiceMode,
    ) -> Priority {
        let mut value = 0.0;
        let mut count = 0;

        if mode == AttractorChoiceMode::WorstChoice {
            value = f64::MAX;
        }

        for choice in choices {
            if self.non_self_loop_mass[choice] == 0.0 {
                continue;
            }
            count += 1;
            let new_value = self.known_mass[choice] / self.non_self_loop_mass[choice];
            match mode {
                AttractorChoiceMode::BestChoice => {
                    if new_value > value {
                        value = new_value;
                    }
                }
                AttractorChoiceMode::WorstChoice => {
                    if new_value < value {
                        value = new_value;
                    }
                }
                AttractorChoiceMode::AverageChoice => value += new_value,
            }
        }
        if mode == AttractorChoiceMode::AverageChoice && count > 0 {
            value = value / count as f64;
        }
        if mode == AttractorChoiceMode::WorstChoice && count == 0 {
            value = 0.0;
        }

        Priority::new(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::mecs::Mecs;
    use crate::sccs::{ExcludeStatesAndChoices, SccEntryIndex, SccIndex, Sccs};
    use crate::sub_model::{Attractor, AttractorChoiceMode, StateOrdering};
    use probabilistic_models::base_model::Mdp;
    use probabilistic_models::traits::ReadStateSpace;
    use probabilistic_models::{
        BranchIndex, ChoiceIndex, Model, PredecessorIndex, StateIndex, mdp,
    };
    use typed_index_collections::{Index, To1};

    const ALL_MODES: [AttractorChoiceMode; 3] = [
        AttractorChoiceMode::AverageChoice,
        AttractorChoiceMode::WorstChoice,
        AttractorChoiceMode::BestChoice,
    ];

    pub fn test_ordering(
        mdp: Mdp<StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>,
        sccs: Vec<Vec<StateIndex<usize>>>,
        compute_mecs: bool,
        scc_index: usize,
        mode: AttractorChoiceMode,
        target_ordering: &[StateIndex<usize>],
    ) {
        let model = Model::new(mdp).compute_predecessors::<PredecessorIndex<usize>>();
        let scc_count = sccs.len();
        let sccs: Sccs<_, SccEntryIndex<usize>, _> = Sccs::new(sccs, model.states().len());
        let mecs = if compute_mecs {
            Mecs::compute(
                &model,
                ExcludeStatesAndChoices::new(
                    To1::with_entries(vec![false; model.states().len()]),
                    To1::with_entries(vec![false; model.choices().len()]),
                ),
            )
        } else {
            Mecs::empty()
        };
        let values = To1::with_entries(vec![0.0; model.states().len()]);
        let attractor = Attractor::new(mode);
        let mut context = attractor.create_context(&model);
        let mut compute = |scc_index: usize| {
            let mut to_old_state_index: To1<StateIndex<usize>, _> = To1::new();
            attractor.compute_ordering(
                &model,
                sccs.scc(SccIndex::from_raw(scc_index)),
                &mecs,
                &values,
                &(),
                &mut context,
                &mut to_old_state_index,
            );
            to_old_state_index
        };

        for other_scc_index in (0..scc_count).filter(|&i| i != scc_index) {
            compute(other_scc_index);
        }
        let ordering = compute(scc_index);

        // Every state of the SCC that is not merged away must occur exactly once
        let mut expected_states: Vec<_> = sccs
            .scc(SccIndex::from_raw(scc_index))
            .states()
            .filter(|&state| !mecs.is_merged_away(state))
            .collect();
        let mut actual_states = ordering.entries().to_vec();
        expected_states.sort();
        actual_states.sort();
        assert_eq!(actual_states, expected_states);

        let target_ordering: Vec<_> = target_ordering
            .iter()
            .map(|&state| mecs.representative(state).unwrap_or(state))
            .collect();
        assert_eq!(ordering.entries(), target_ordering);
    }

    #[test]
    fn simple() {
        for mode in ALL_MODES {
            mdp!(mdp = {s0 -> 1.0: s1, s1 -> 1.0: s2, s2 -> 0.7: s0 & 0.3: s3, s3 -> deadlock});
            let sccs = vec![vec![s0, s1, s2], vec![s3]];
            test_ordering(mdp, sccs, false, 0, mode, &[s2, s1, s0]);
        }
    }

    #[test]
    fn singleton_at_non_zero_index() {
        for mode in ALL_MODES {
            for compute_mecs in [false, true] {
                mdp!(mdp = {
                    s0 -> 1.0: s1,
                    s1 -> 0.5: s1 & 0.5: s2,
                    s2 -> deadlock
                });
                let sccs = vec![vec![s0], vec![s1], vec![s2]];
                test_ordering(mdp, sccs, compute_mecs, 1, mode, &[s1]);
            }
        }
    }

    #[test]
    fn trivial_singleton() {
        for mode in ALL_MODES {
            for compute_mecs in [false, true] {
                mdp!(mdp = {s0 -> 0.5: s1 & 0.5: s2, s1 -> deadlock, s2 -> deadlock});
                let sccs = vec![vec![s0], vec![s1], vec![s2]];
                test_ordering(mdp.clone(), sccs.clone(), compute_mecs, 0, mode, &[s0]);
                test_ordering(mdp, sccs, compute_mecs, 1, mode, &[s1]);
            }
        }
    }

    #[test]
    fn singleton_with_only_self_loop() {
        for mode in ALL_MODES {
            for compute_mecs in [false, true] {
                mdp!(mdp = {s0 -> 1.0: s1, s1 -> 1.0: s1});
                let sccs = vec![vec![s0], vec![s1]];
                test_ordering(mdp, sccs, compute_mecs, 1, mode, &[s1]);
            }
        }
    }

    #[test]
    fn singleton_with_compound_loop() {
        for mode in ALL_MODES {
            for compute_mecs in [false, true] {
                mdp!(mdp = {s0 -> 1.0: s1, s1 -> 0.3: s1 & 0.7: s1, s1 -> 0.4: s1 & 0.6: s1});
                let sccs = vec![vec![s0], vec![s1]];
                test_ordering(mdp, sccs, compute_mecs, 1, mode, &[s1]);
            }
        }
    }

    #[test]
    fn bottom_mec() {
        for mode in ALL_MODES {
            mdp!(mdp = {s0 -> 1.0: s1, s1 -> 1.0: s2, s2 -> 1.0: s1});
            let sccs = vec![vec![s0], vec![s1, s2]];
            // Without MECs, no state has an exit, so the tie is broken by the higher index
            test_ordering(mdp, sccs, false, 1, mode, &[s2, s1]);

            mdp!(mdp = {s0 -> 1.0: s1, s1 -> 1.0: s2, s2 -> 1.0: s1});
            let sccs = vec![vec![s0], vec![s1, s2]];
            // With MECs, only the representative is part of the ordering
            test_ordering(mdp, sccs, true, 1, mode, &[s1]);
        }
    }

    #[test]
    fn collapsed_mec() {
        // Verifies that all predecessors of the MEC {s1, s2} are visited.
        for mode in ALL_MODES {
            mdp!(mdp = {
                s0 -> 1.0: s1,
                s1 -> 1.0: s2,
                s2 -> 1.0: s1,
                s2 -> 0.5: s5 & 0.5: s4,
                s3 -> 1.0: s2,
                s4 -> 0.3: s5 & 0.35: s0 & 0.35: s3,
                s5 -> deadlock
            });
            let sccs = vec![vec![s0, s1, s2, s3, s4], vec![s5]];
            // The tie between s0 and s3 is broken by the higher index
            test_ordering(mdp, sccs, true, 0, mode, &[s1, s3, s0, s4]);
        }
    }

    #[test]
    fn circle_follows_dominant_direction() {
        for mode in ALL_MODES {
            mdp!(mdp = {
                s0 -> 0.9: s1 & 0.1: s3,
                s1 -> 0.9: s2 & 0.1: s0,
                s2 -> 0.8: s3 & 0.1: s1 & 0.1: s4,
                s3 -> 0.9: s0 & 0.1: s2,
                s4 -> deadlock
            });
            let sccs = vec![vec![s0, s1, s2, s3], vec![s4]];
            test_ordering(mdp, sccs, false, 0, mode, &[s2, s1, s0, s3]);
        }
    }

    #[test]
    fn self_loop_scales_exit_probability() {
        // s0 is chosen first, even though s1 has higher exit probability (but s0 has a self loop)
        for mode in ALL_MODES {
            mdp!(mdp = {
                s0 -> 0.5: s3 & 0.5: s1,
                s1 -> 0.2: s3 & 0.7: s1 & 0.1: s2,
                s2 -> 0.5: s0 & 0.5: s1,
                s3 -> deadlock
            });
            let sccs = vec![vec![s0, s1, s2], vec![s3]];
            test_ordering(mdp, sccs, false, 0, mode, &[s1, s0, s2]);
        }
    }

    #[test]
    fn choice_modes() {
        // Initial known mass per choice: s0: 0.9 and 0.0, s1: 0.7 and 0.5, s2: 0.55. Therefore,
        for (mode, target) in [
            (AttractorChoiceMode::BestChoice, [0, 2, 1]),
            (AttractorChoiceMode::AverageChoice, [1, 0, 2]),
            (AttractorChoiceMode::WorstChoice, [2, 1, 0]),
        ] {
            mdp!(mdp = {
                s0 -> 0.9: s3 & 0.1: s1,
                s0 -> 1.0: s1,
                s1 -> 0.7: s3 & 0.3: s2,
                s1 -> 0.5: s3 & 0.5: s2,
                s2 -> 0.55: s3 & 0.45: s0,
                s3 -> deadlock
            });
            let sccs = vec![vec![s0, s1, s2], vec![s3]];
            let target = target.map(StateIndex::from_raw);
            test_ordering(mdp, sccs, false, 0, mode, &target);
        }
    }

    #[test]
    fn predecessor_outside_scc() {
        // s0 and s4 lead into the SCC {s1, s2}, but must not become part of its ordering.
        for mode in ALL_MODES {
            mdp!(mdp = {
                s0 -> 1.0: s1,
                s1 -> 0.5: s2 & 0.5: s3,
                s2 -> 1.0: s1,
                s3 -> deadlock,
                s4 -> 0.5: s2 & 0.5: s3
            });
            let sccs = vec![vec![s0], vec![s4], vec![s1, s2], vec![s3]];
            test_ordering(mdp, sccs, false, 2, mode, &[s1, s2]);
        }
    }

    #[test]
    fn updated_priority_overtakes_stale_entry() {
        for mode in ALL_MODES {
            mdp!(mdp = {
                s0 -> 0.9: s4 & 0.1: s2,
                s1 -> 0.8: s4 & 0.2: s3,
                s2 -> 0.3: s0 & 0.3: s1 & 0.4: s3,
                s3 -> 0.5: s4 & 0.5: s2,
                s4 -> deadlock
            });
            let sccs = vec![vec![s0, s1, s2, s3], vec![s4]];
            test_ordering(mdp, sccs, false, 0, mode, &[s0, s1, s2, s3]);
        }
    }
}
