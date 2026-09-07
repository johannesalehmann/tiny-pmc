use crate::dominated_by::DominatedByRelation;
use crate::sccs::{SccDependencyIndex, SccEntryIndex, SccIndex, Sccs};
use crate::value_iteration::sub_model::{SubModelContext, build_sub_model};
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::{ReadAtomicPropositions, ReadPredecessors, ReadStateSpace};
use probabilistic_models::{BranchIndex, ChoiceIndex, StateIndex};
use std::time::Duration;
use typed_index_collections::{Index, RawIndex, To1};

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
    eps: f64,
) -> To1<<M as ReadStateSpace>::StateIdx, f64> {
    let mut precomputation_time = Duration::default();
    let mut build_time = Duration::default();
    let mut value_iteration_time = Duration::default();

    let mut precomputation_start = std::time::Instant::now();
    let s0_max = super::precomputation::s0_max(model, goal);
    let s1_max = super::precomputation::s1_max(model, goal);

    let mut values = To1::with_capacity(model.states().len());
    for state in model.states() {
        values.add_checked(state, if s1_max[state] { 1.0 } else { 0.0 });
    }

    let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> =
        Sccs::compute(model, Some((s0_max, s1_max)));

    // TODO: Only count non-singleton SCCs for the longest chain. That way, we get a bit more
    //  precision budget for those, as singleton SCCs can be solved exactly (except for floating-
    //  point rounding).
    let longest_chain = sccs
        .compute_dependencies::<SccDependencyIndex<usize>, _>(model)
        .longest_chain();
    precomputation_time += precomputation_start.elapsed();

    // TODO: Perhaps allocate precision depending on SCC size? And what about SCCs that are not
    //  part of the longest SCC chain? And if an upstream SCC is solved with higher precision than
    //  planned, we can allocate the extra budget to the downstream SCCs.
    // TODO: Is this approach actually correct when using a relative error criterion?
    let precision_per_scc = 2.0 * eps * (1.0 / longest_chain as f64);

    let mut subgame_construction_context = SubModelContext::new(model);
    // TODO: Determine size of largest SCC (or even subgame?) and use that as buffer size instead?
    let mut subgame_values = vec![0.0; model.states().len()];
    let mut subgame_upper_bound = vec![0.0; model.states().len()];
    let mut core_start = std::time::Instant::now();
    for scc in sccs.reverse_topological_ordering() {
        if sccs.entries(scc).len() == 1 {
            let entry = sccs.entries(scc).into_iter().next().unwrap();
            let state = sccs.state_of_entry(entry);

            let mut best_value = 0.0;
            for choice in model.choices_of_state(state) {
                let choice_value = evaluate_choice(model, &values, state, choice);
                if choice_value > best_value {
                    best_value = choice_value;
                }
            }
            values[state] = best_value;
        } else {
            let states = sccs.entries(scc).len();
            let mut choices = 0;
            let mut branches = 0;
            for entry in sccs.entries(scc) {
                let state = sccs.state_of_entry(entry);
                let entry_choices = model.choices_of_state(state);
                choices += entry_choices.len();
                for choice in entry_choices {
                    branches += model.branches_of_choice(choice).len();
                }
            }

            if states < 256 && choices < 256 && branches < 256 {
                build_and_solve_submodel::<StateIndex<u8>, ChoiceIndex<u8>, BranchIndex<u8>, _>(
                    model,
                    &mut build_time,
                    &mut value_iteration_time,
                    &mut values,
                    &sccs,
                    precision_per_scc,
                    &mut subgame_construction_context,
                    &mut subgame_values,
                    &mut subgame_upper_bound,
                    scc,
                );
            } else if states < 256 * 256 && choices < 256 * 256 && branches < 256 * 256 {
                build_and_solve_submodel::<StateIndex<u16>, ChoiceIndex<u16>, BranchIndex<u16>, _>(
                    model,
                    &mut build_time,
                    &mut value_iteration_time,
                    &mut values,
                    &sccs,
                    precision_per_scc,
                    &mut subgame_construction_context,
                    &mut subgame_values,
                    &mut subgame_upper_bound,
                    scc,
                );
            } else if states < 256 * 256 * 256 * 256
                && choices < 256 * 256 * 256 * 256
                && branches < 256 * 256 * 256 * 256
            {
                build_and_solve_submodel::<StateIndex<u32>, ChoiceIndex<u32>, BranchIndex<u32>, _>(
                    model,
                    &mut build_time,
                    &mut value_iteration_time,
                    &mut values,
                    &sccs,
                    precision_per_scc,
                    &mut subgame_construction_context,
                    &mut subgame_values,
                    &mut subgame_upper_bound,
                    scc,
                );
            } else {
                // Using u64 instead of usize here would not help here, as on 32-bit platforms, the
                // underlying indexing operation would fail regardless (because it converts the
                // index type into a usize).
                build_and_solve_submodel::<
                    StateIndex<usize>,
                    ChoiceIndex<usize>,
                    BranchIndex<usize>,
                    _,
                >(
                    model,
                    &mut build_time,
                    &mut value_iteration_time,
                    &mut values,
                    &sccs,
                    precision_per_scc,
                    &mut subgame_construction_context,
                    &mut subgame_values,
                    &mut subgame_upper_bound,
                    scc,
                );
            }
        }
    }
    println!("Total core time: {:?}", core_start.elapsed());
    println!(
        "Precomputation time: {:?}, sub model build time: {:?}, value iteration time: {:?}",
        precomputation_time, build_time, value_iteration_time
    );
    values
}

// TODO: This function signature is a mess
fn build_and_solve_submodel<NewSI: Index, NewCI: Index, NewBI: Index, M: ReadStateSpace>(
    model: &M,
    build_time: &mut Duration,
    value_iteration_time: &mut Duration,
    values: &mut To1<M::StateIdx, f64>,
    sccs: &Sccs<SccIndex<usize>, SccEntryIndex<usize>, M::StateIdx>,
    precision_per_scc: f64,
    subgame_construction_context: &mut SubModelContext<M::StateIdx>,
    subgame_values: &mut Vec<f64>,
    subgame_upper_bound: &mut Vec<f64>,
    scc: SccIndex<usize>,
) {
    let start_submodel = std::time::Instant::now();
    let sub_model = build_sub_model::<_, _, _, NewSI, NewCI, NewBI>(
        model,
        scc,
        &sccs,
        &DominatedByRelation::empty(),
        &values,
        subgame_construction_context,
    );
    *build_time += start_submodel.elapsed();

    let start_solve = std::time::Instant::now();
    solve_subgame_via_ovi(
        &sub_model.mdp,
        &sub_model.choice_exit_values,
        precision_per_scc,
        subgame_values,
        subgame_upper_bound,
    );
    *value_iteration_time += start_solve.elapsed();

    for new_state in sub_model.mdp.states() {
        values[sub_model.to_old_state_index[new_state]] =
            subgame_values[new_state.raw().as_usize()];
    }
}

fn evaluate_choice<M: ReadStateSpace>(
    model: &M,
    values: &To1<M::StateIdx, f64>,
    state: M::StateIdx,
    choice: M::ChoiceIdx,
) -> f64 {
    let mut to_self = 0.0;
    let mut exit_value = 0.0;
    for branch in model.branches_of_choice(choice) {
        let destination = model.branch_destination(branch);
        let p = model.branch_probability(branch);
        if destination == state {
            to_self += p;
        } else {
            exit_value += p * values[destination];
        }
    }
    if to_self == 1.0 {
        0.0
    } else {
        exit_value / (1.0 - to_self)
    }
}

fn solve_subgame_via_ovi<NewSI: Index, NewCI: Index, NewBI: Index>(
    mdp: &Mdp<NewSI, NewCI, NewBI>,
    choice_exit_values: &To1<NewCI, f64>,
    mut eps: f64,
    values: &mut Vec<f64>,
    upper_bound: &mut Vec<f64>,
) {
    for state in mdp.states() {
        values[state.raw().as_usize()] = 0.0;
    }

    let initial_eps = eps;
    loop {
        subgame_value_iteration(mdp, choice_exit_values, eps, values);

        for state in mdp.states() {
            upper_bound[state.raw().as_usize()] = match values[state.raw().as_usize()] {
                0.0 => 0.0,
                v => (v * (1.0 + initial_eps)).min(1.0),
            }
        }

        match verify_subgame_optimistic(mdp, choice_exit_values, 2.0 * eps, values, upper_bound) {
            OptimisticValueIterationResult::UpperBoundVerified => {
                for state in mdp.states() {
                    values[state.raw().as_usize()] = 0.5
                        * (values[state.raw().as_usize()] + upper_bound[state.raw().as_usize()]);
                }
                break;
            }
            OptimisticValueIterationResult::UpperBoundRefuted { error } => {
                eps = error * 0.5;
            }
        }
    }
}

fn subgame_value_iteration<NewSI: Index, NewCI: Index, NewBI: Index>(
    mdp: &Mdp<NewSI, NewCI, NewBI>,
    choice_exit_values: &To1<NewCI, f64>,
    eps: f64,
    values: &mut Vec<f64>,
) {
    loop {
        let mut converged = true;
        // Iterate states manually instead of relying on built-in functions such as
        // choices_of_state() because this is about 15% faster:
        let mut current_choice = NewCI::default();
        let mut current_branch = NewBI::default();
        let mut choices = mdp.choice_to_branch.entries_raw().iter();
        let mut branches = mdp
            .branch_probabilities
            .iter()
            .zip(mdp.branch_destinations.iter());
        let mut choice_exit_values = choice_exit_values.iter();

        for (state, &last_choice) in mdp.state_to_choice.entries_raw().iter().enumerate() {
            let state = NewSI::from_raw(NewSI::RawType::from_usize(state));
            let mut best_value = 0.0;
            while current_choice < last_choice {
                let last_branch = *choices.next().unwrap();
                let mut value = *choice_exit_values.next().unwrap();
                while current_branch < last_branch {
                    let (probability, destination) = branches.next().unwrap();
                    value += probability * values[destination.raw().as_usize()];
                    current_branch += NewBI::RawType::one();
                }
                if value >= best_value {
                    best_value = value;
                }
                current_choice += NewCI::RawType::one();
            }

            if converged {
                let absolute_error = best_value - values[state.raw().as_usize()];
                let relative_error = absolute_error / best_value;
                if relative_error >= eps {
                    converged = false;
                }
            }
            values[state.raw().as_usize()] = best_value;
        }
        if converged {
            break;
        }
    }
}

fn verify_subgame_optimistic<NewSI: Index, NewCI: Index, NewBI: Index>(
    mdp: &Mdp<NewSI, NewCI, NewBI>,
    choice_exit_values: &To1<NewCI, f64>,
    eps: f64,
    values: &mut Vec<f64>,
    upper_bound: &mut Vec<f64>,
) -> OptimisticValueIterationResult {
    let verification_steps = (1.0 / eps).max(1.0) as usize;
    let mut error: f64 = 0.0;
    for _ in 0..verification_steps {
        let mut all_up = true;
        let mut all_down = true;
        error = 0.0;
        let mut current_choice = NewCI::default();
        let mut current_branch = NewBI::default();
        let mut choices = mdp.choice_to_branch.entries_raw().iter();
        let mut branches = mdp
            .branch_probabilities
            .iter()
            .zip(mdp.branch_destinations.iter());
        let mut choice_exit_values = choice_exit_values.iter();
        for (state, &last_choice) in mdp.state_to_choice.entries_raw().iter().enumerate() {
            let state = NewSI::from_raw(NewSI::RawType::from_usize(state));
            let state_raw = state.raw().as_usize();
            let mut new_lower_value = 0.0;
            let mut new_upper_value = 0.0;

            while current_choice < last_choice {
                let last_branch = *choices.next().unwrap();
                let exit_value = *choice_exit_values.next().unwrap();
                let mut lower_value = exit_value;
                let mut upper_value = exit_value;
                while current_branch < last_branch {
                    let (probability, destination) = branches.next().unwrap();
                    lower_value += probability * values[destination.raw().as_usize()];
                    upper_value += probability * upper_bound[destination.raw().as_usize()];
                    current_branch += NewBI::RawType::one();
                }
                if lower_value >= new_lower_value {
                    new_lower_value = lower_value;
                }
                if upper_value >= new_upper_value {
                    new_upper_value = upper_value;
                }
                current_choice += NewCI::RawType::one();
            }

            if new_lower_value > 0.0 {
                error = error.max((new_lower_value - values[state_raw]) / new_lower_value);
            }
            values[state_raw] = new_lower_value;
            if new_upper_value < upper_bound[state_raw] {
                all_up = false;
                upper_bound[state_raw] = new_upper_value;
            } else if new_upper_value > upper_bound[state_raw] {
                all_down = false;
            }

            if new_upper_value < new_lower_value {
                return OptimisticValueIterationResult::UpperBoundRefuted { error };
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
