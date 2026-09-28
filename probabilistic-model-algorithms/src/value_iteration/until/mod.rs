use crate::state_description::StateDescription;
use probabilistic_models::annotations::{AtomicPropositions, TypedAnnotation};
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::StateSet;
use probabilistic_models::traits::{ReadAtomicPropositions, ReadInitialStates, ReadStateSpace};
use probabilistic_models::{InitialStates, Model};
use typed_index_collections::{Index, RawIndex, To1};

/// Builds a restricted model for checking `restriction U goal`. It contains only the reachable
/// fragment that satisfies `restriction` and a goal and sink state. The only atomic proposition
/// is the one marking goal, which is named `goal_name`.
pub fn rebuild_model_for_until<
    SI: Index,
    APEI: Index,
    M: ReadStateSpace<StateIndex = SI>
        + ReadAtomicPropositions<StateIdx = SI>
        + ReadInitialStates<StateIdx = SI>,
>(
    model: &M,
    restriction: &StateDescription<M>,
    goal: &StateDescription<M>,
    goal_name: &str,
) -> (
    Model<
        Mdp<SI, M::ChoiceIndex, M::BranchIndex>,
        InitialStates<SI>,
        (),
        (),
        (),
        AtomicPropositions<M::APIdx, SI, APEI>,
        (),
        (),
        (),
        (),
    >,
    M::APIdx,
) {
    let is_maybe_state = |state: SI| restriction.is_set(state) && !goal.is_set(state);

    // Explore all states that satisfy `restriction` but not `goal` depth-first.
    let mut old_to_new: To1<SI, Option<SI>> = To1::with_entries(vec![None; model.states().len()]);
    let mut new_to_old: To1<SI, SI> = To1::new();
    let mut open_list: Vec<SI> = model.initial_states().iter().collect();
    while let Some(state) = open_list.pop() {
        if old_to_new[state].is_some() || !is_maybe_state(state) {
            continue;
        }
        old_to_new[state] = Some(new_to_old.add(state));
        open_list.extend(
            model
                .successors_of_state(state)
                .filter(|&successor| old_to_new[successor].is_none()),
        );
    }

    // Rebuild the model with the goal and sink state at the end.
    let goal_state = SI::from_raw(SI::RawType::from_usize(new_to_old.len()));
    let sink_state = goal_state + SI::RawType::one();
    let state_count = new_to_old.len() + 2;

    let mut mdp = Mdp::default();
    for (new_state, &state) in new_to_old.enumerate() {
        mdp.add_state(new_state);
        for choice in model.choices_of_state(state) {
            mdp.add_choice();
            // Branches that are redirected to the goal or sink state are merged into one branch.
            let mut to_goal = None;
            let mut to_sink = None;
            for branch in model.branches_of_choice(choice) {
                let destination = model.branch_destination(branch);
                let probability = model.branch_probability(branch);
                if let Some(target) = old_to_new[destination] {
                    mdp.add_branch(probability, target);
                } else if goal.is_set(destination) {
                    *to_goal.get_or_insert(0.0) += probability;
                } else {
                    // If destination is not in old_to_new, it is guaranteed not to satisfy
                    // `restriction`.
                    debug_assert!(!restriction.is_set(destination));
                    *to_sink.get_or_insert(0.0) += probability;
                }
            }
            if let Some(probability) = to_goal {
                mdp.add_branch(probability, goal_state);
            }
            if let Some(probability) = to_sink {
                mdp.add_branch(probability, sink_state);
            }
        }
    }
    for absorbing_state in [goal_state, sink_state] {
        mdp.add_state(absorbing_state);
        mdp.add_choice_from_slice(&[(1.0, absorbing_state)]);
    }

    let mut goal_annotation =
        TypedAnnotation::with_identity_distribution_and_entries(To1::with_entries(vec![
            false;
            state_count
        ]));
    goal_annotation[goal_state] = true;
    let mut atomic_propositions = AtomicPropositions::new();
    let new_goal_index = atomic_propositions.add_entry(goal_name.to_string(), goal_annotation);

    let mut initial = InitialStates::with_entries(vec![false; state_count]);
    for state in model.initial_states().iter() {
        let new_state = match old_to_new[state] {
            Some(new_state) => new_state,
            None if goal.is_set(state) => goal_state,
            None => sink_state,
        };
        initial[new_state] = true;
    }

    (
        Model {
            base: mdp,
            initial,
            choice_labels: (),
            branch_labels: (),
            observations: (),
            atomic_propositions,
            rewards: (),
            annotations: (),
            state_valuations: (),
            predecessors: (),
        },
        new_goal_index,
    )
}

#[cfg(test)]
mod tests {
    use super::rebuild_model_for_until;
    use crate::state_description::StateDescription;
    use probabilistic_models::annotations::{AtomicPropositions, TypedAnnotation};
    use probabilistic_models::base_model::Mdp;
    use probabilistic_models::traits::ReadStateSpace;
    use probabilistic_models::{
        AnnotationEntryIndex, AnnotationIndex, BranchIndex, ChoiceIndex, InitialStates, Model,
        StateIndex, mdp,
    };
    use typed_index_collections::{Index, To1};

    type SI = StateIndex<usize>;
    type TestMdp = Mdp<SI, ChoiceIndex<usize>, BranchIndex<usize>>;
    type TestAps = AtomicPropositions<AnnotationIndex<usize>, SI, AnnotationEntryIndex<usize>>;

    fn state_set(state_count: usize, states: &[SI]) -> To1<SI, bool> {
        let mut set = To1::with_entries(vec![false; state_count]);
        for &state in states {
            set[state] = true;
        }
        set
    }

    fn check_rebuild(
        mdp: TestMdp,
        initial: &[SI],
        restriction: &[SI],
        goal: &[SI],
        expected: TestMdp,
        expected_initial: &[SI],
    ) {
        let state_count = mdp.states().len();
        let mut aps = TestAps::new();
        let restriction_ap = aps.add_entry(
            "restriction".to_string(),
            TypedAnnotation::with_identity_distribution_and_entries(
                state_set(state_count, restriction).change_key_type(),
            ),
        );
        let goal_ap = aps.add_entry(
            "goal".to_string(),
            TypedAnnotation::with_identity_distribution_and_entries(
                state_set(state_count, goal).change_key_type(),
            ),
        );
        let model = Model {
            base: mdp,
            initial: state_set(state_count, initial),
            choice_labels: (),
            branch_labels: (),
            observations: (),
            atomic_propositions: aps,
            rewards: (),
            annotations: (),
            state_valuations: (),
            predecessors: (),
        };

        let (result, result_goal_ap) = rebuild_model_for_until::<_, AnnotationEntryIndex<usize>, _>(
            &model,
            &StateDescription::AtomicProposition {
                ap_index: restriction_ap,
                model: &model,
            },
            &StateDescription::AtomicProposition {
                ap_index: goal_ap,
                model: &model,
            },
            "target",
        );

        let expected_state_count = expected.states().len();
        let mut expected_aps = TestAps::new();
        expected_aps.add_entry(
            "target".to_string(),
            TypedAnnotation::with_identity_distribution_and_entries(
                state_set(
                    expected_state_count,
                    &[SI::from_raw(expected_state_count - 2)],
                )
                .change_key_type(),
            ),
        );
        let expected_initial: InitialStates<SI> = state_set(expected_state_count, expected_initial);

        assert_eq!(result.base, expected);
        assert_eq!(result.initial, expected_initial);
        assert_eq!(result.atomic_propositions, expected_aps);
        assert_eq!(
            result.atomic_propositions.name(result_goal_ap),
            Some("target")
        );
    }

    #[test]
    fn simple_model() {
        mdp!(mdp = {
            s0 -> 0.5: s1 & 0.5: s3,
            s1 -> 0.3: s0 & 0.7: s2,
            s2 -> 1.0: s2,
            s3 -> 1.0: s3
        });
        mdp!(expected = {
            n0 -> 0.5: n1 & 0.5: sink,
            n1 -> 0.3: n0 & 0.7: goal,
            goal -> 1.0: goal,
            sink -> 1.0: sink
        });
        check_rebuild(mdp, &[s0], &[s0, s1], &[s2], expected, &[n0]);
    }

    #[test]
    fn initial_state_satisfies_goal() {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 0.5: s0 & 0.5: s2,
            s2 -> 1.0: s2
        });
        mdp!(expected = {
            goal -> 1.0: goal,
            sink -> 1.0: sink
        });
        check_rebuild(mdp, &[s0], &[s0, s1], &[s0], expected, &[goal]);
    }

    #[test]
    fn initial_goal_state_and_initial_state_with_structure() {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 0.5: s2 & 0.5: s1,
            s2 -> 1.0: s0
        });
        mdp!(expected = {
            n0 -> 0.5: n1 & 0.5: n0,
            n1 -> 1.0: goal,
            goal -> 1.0: goal,
            sink -> 1.0: sink
        });
        check_rebuild(mdp, &[s0, s1], &[s1, s2], &[s0], expected, &[n0, goal]);
    }

    #[test]
    fn merge_branches_to_goal_and_sink() {
        mdp!(mdp = {
            s0 -> 0.1: g0 & 0.2: f0 & 0.1: s0 & 0.3: g1 & 0.3: f1,
            s0 -> 1.0: g0,
            g0 -> 1.0: g0,
            g1 -> 1.0: g1,
            f0 -> 1.0: f0,
            f1 -> 1.0: f1
        });
        mdp!(expected = {
            n0 -> 0.1: n0 & 0.4: goal & 0.5: sink,
            n0 -> 1.0: goal,
            goal -> 1.0: goal,
            sink -> 1.0: sink
        });
        check_rebuild(mdp, &[s0], &[s0], &[g0, g1], expected, &[n0]);
    }

    #[test]
    fn non_zero_initial_state() {
        mdp!(mdp = {
            s0 -> 1.0: s0,
            s1 -> 0.5: s0 & 0.5: s2,
            s2 -> 1.0: s2
        });
        mdp!(expected = {
            n0 -> 0.5: n1 & 0.5: goal,
            n1 -> 1.0: n1,
            goal -> 1.0: goal,
            sink -> 1.0: sink
        });
        check_rebuild(mdp, &[s1], &[s0, s1], &[s2], expected, &[n0]);
    }

    #[test]
    fn unreachable_states_are_pruned() {
        mdp!(mdp = {
            s0 -> 0.5: s1 & 0.5: s3,
            s1 -> 1.0: s2,
            s2 -> 1.0: s0,
            s3 -> 1.0: s4,
            s4 -> 1.0: s4,
            s5 -> 1.0: s0
        });
        mdp!(expected = {
            n0 -> 0.5: goal & 0.5: sink,
            goal -> 1.0: goal,
            sink -> 1.0: sink
        });
        check_rebuild(mdp, &[s0], &[s0, s2, s4, s5], &[s1], expected, &[n0]);
    }

    #[test]
    fn initial_state_satisfies_neither() {
        mdp!(mdp = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s1
        });
        mdp!(expected = {
            goal -> 1.0: goal,
            sink -> 1.0: sink
        });
        check_rebuild(mdp, &[s0], &[s1], &[], expected, &[sink]);
    }

    #[test]
    fn deadlock_state() {
        mdp!(mdp = {
            s0 -> 0.5: s1 & 0.5: s2,
            s1 -> deadlock,
            s2 -> 1.0: s2
        });
        mdp!(expected = {
            n0 -> 0.5: n1 & 0.5: goal,
            n1 -> deadlock,
            goal -> 1.0: goal,
            sink -> 1.0: sink
        });
        check_rebuild(mdp, &[s0], &[s0, s1], &[s2], expected, &[n0]);
    }

    #[test]
    fn choice_only_reaching_goal_and_sink() {
        mdp!(mdp = {
            s0 -> 0.4: s1 & 0.6: s2,
            s1 -> 1.0: s1,
            s2 -> 1.0: s2
        });
        mdp!(expected = {
            n0 -> 0.6: goal & 0.4: sink,
            goal -> 1.0: goal,
            sink -> 1.0: sink
        });
        check_rebuild(mdp, &[s0], &[s0], &[s2], expected, &[n0]);
    }

    #[test]
    fn goal_takes_precedence_over_restriction() {
        mdp!(mdp = {
            s0 -> 0.5: s1 & 0.5: s2,
            s1 -> 1.0: s0,
            s2 -> 1.0: s2
        });
        mdp!(expected = {
            n0 -> 0.5: goal & 0.5: sink,
            goal -> 1.0: goal,
            sink -> 1.0: sink
        });
        check_rebuild(mdp, &[s0], &[s0, s1], &[s1], expected, &[n0]);
    }
}
