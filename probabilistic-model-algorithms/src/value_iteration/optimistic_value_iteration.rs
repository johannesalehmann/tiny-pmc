use crate::sccs::{ExclusionList, SccEntryIndex, SccIndex, Sccs};
use crate::value_iteration::min_max::{Maximiser, ValueComparator};
use probabilistic_models::traits::{ReadAtomicPropositions, ReadPredecessors, ReadStateSpace};
use typed_index_collections::{Index, To1};

// TODO: Enforce that this only works for single-player models?
pub fn optimistic_value_iteration_max<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    goal: <M as ReadAtomicPropositions>::APIdx,
    mut eps: f64,
) -> To1<<M as ReadStateSpace>::StateIdx, f64> {
    let min_max = Maximiser::default();

    let p0p1_states = super::precomputation::compute_p_0_and_p1_states(model, min_max, goal);
    println!(
        "Found {} states with probability 0, {} with probability 1",
        p0p1_states.p0_states().into_iter().count(),
        p0p1_states.p1_states().into_iter().count(),
    );

    let mut values = To1::with_capacity(model.states().len());
    let mut upper_bound = To1::with_entries(vec![0.0; model.states().len()]);
    let mut target_states = Vec::new();
    // TODO: The p1 states already include every state of the target set. Thus, it is likely not
    //  necessary to separately track the set of target states
    for state in model.states() {
        if model.is_atomic_proposition_set(state, goal) {
            target_states.push(state);
            values.add_checked(state, 1.0);
        } else {
            if p0p1_states.is_state_p1(state) {
                values.add_checked(state, 1.0);
            } else {
                values.add_checked(state, 0.0);
            }
        }
    }

    let excluded = ExclusionList::new(&target_states);

    let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> =
        Sccs::compute(model, &excluded, Some(p0p1_states));

    let mut initial_eps = eps;
    loop {
        println!("eps={}:", eps);
        let start_vi = std::time::Instant::now();
        super::value_iteration_internal(model, eps, min_max, &mut values, &sccs);
        println!("Value iteration in {:?}", start_vi.elapsed());

        for i in model.states() {
            upper_bound[i] = match values[i] {
                0.0 => 0.0,
                v => (v + initial_eps).min(1.0),
            }
        }

        let start_verification = std::time::Instant::now();
        let is_upper_bound =
            verify_optimistic(model, eps, &mut values, &mut upper_bound, &sccs, min_max);
        println!("Verification in {:?}", start_verification.elapsed());

        match is_upper_bound {
            OptimisticValueIterationResult::UpperBoundVerified => {
                println!("Upper bound candidate verified!");
                for i in model.states() {
                    values[i] = 0.5 * (values[i] + upper_bound[i]);
                }
                break values;
            }
            OptimisticValueIterationResult::UpperBoundRefuted { error } => {
                eps = error * 0.5;
            }
        }
    }
}

fn verify_optimistic<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<StateIdx = <M as ReadStateSpace>::StateIdx>,
    MinMax: ValueComparator<Model = M>,
    SccIdx: Index,
    SccEntryIdx: Index,
>(
    model: &M,
    eps: f64,
    values: &mut To1<<M as ReadStateSpace>::StateIdx, f64>,
    upper_bound: &mut To1<<M as ReadStateSpace>::StateIdx, f64>,
    sccs: &Sccs<SccIdx, SccEntryIdx, <M as ReadStateSpace>::StateIdx>,
    min_max: MinMax,
) -> OptimisticValueIterationResult {
    let verification_steps = (1.0 / eps).max(1.0) as usize;
    let mut error: f64 = 0.0;
    for _ in 0..verification_steps {
        let mut all_up = true;
        let mut all_down = true;
        error = 0.0;
        for scc_index in sccs.reverse_topological_ordering() {
            for entry in sccs.entries(scc_index) {
                let state = sccs.entry_to_state(entry);

                let mut new_lower_value = min_max.neutral_value(state, model);
                let mut new_upper_value = min_max.neutral_value(state, model);

                for choice in model.choices_of_state(state) {
                    let mut lower_value = 0.0;
                    let mut upper_value = 0.0;
                    for branch in model.branches_of_choice(choice) {
                        lower_value += model.branch_probability(branch)
                            * values[model.branch_destination(branch)];
                        upper_value += model.branch_probability(branch)
                            * upper_bound[model.branch_destination(branch)];
                    }
                    if min_max.is_better(state, model, new_lower_value, lower_value) {
                        new_lower_value = lower_value;
                    }
                    if min_max.is_better(state, model, new_upper_value, upper_value) {
                        new_upper_value = upper_value;
                    }
                }
                if model.choices_of_state(state).len() == 0 {
                    new_lower_value = 0.0;
                    new_upper_value = 0.0;
                }

                if new_lower_value > 0.0 {
                    error = error.max(new_lower_value - values[state]);
                }
                values[state] = new_lower_value;
                if new_upper_value < upper_bound[state] {
                    all_up = false;
                    upper_bound[state] = new_upper_value;
                } else if new_upper_value > upper_bound[state] {
                    all_down = false;
                }

                if new_upper_value < new_lower_value {
                    return OptimisticValueIterationResult::UpperBoundRefuted { error };
                }
            }
        }

        if all_down {
            return OptimisticValueIterationResult::UpperBoundVerified;
        } else if all_up {
            return OptimisticValueIterationResult::UpperBoundRefuted { error };
        }
    }
    OptimisticValueIterationResult::UpperBoundRefuted { error }
}

enum OptimisticValueIterationResult {
    UpperBoundVerified,
    UpperBoundRefuted { error: f64 },
}
