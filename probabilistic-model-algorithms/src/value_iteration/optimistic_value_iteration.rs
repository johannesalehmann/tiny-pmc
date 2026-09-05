use crate::dominated_by::DominatedByRelation;
use crate::sccs::{SccDependencyIndex, SccEntryIndex, SccIndex, Sccs};
use crate::value_iteration::sub_model::{SubModelContext, build_sub_model};
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::{ReadAtomicPropositions, ReadPredecessors, ReadStateSpace};
use probabilistic_models::{BranchIndex, ChoiceIndex, StateIndex};
use typed_index_collections::{Index, To1};

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

    // TODO: Perhaps allocate precision depending on SCC size? And what about SCCs that are not
    //  part of the longest SCC chain? And if an upstream SCC is solved with higher precision than
    //  planned, we can allocate the extra budget to the downstream SCCs.
    let precision_per_scc = 2.0 * eps * (1.0 / longest_chain as f64);

    let mut subgame_construction_context = SubModelContext::new(model);
    // TODO: Determine size of largest SCC (or even subgame?) and use that as buffer size instead?
    let mut subgame_values = To1::with_entries(vec![0.0; model.states().len()]);
    let mut subgame_upper_bound = To1::with_entries(vec![0.0; model.states().len()]);
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
            // TODO: Dynamically select sub model index types?
            let sub_model = build_sub_model::<
                _,
                _,
                _,
                StateIndex<usize>,
                ChoiceIndex<usize>,
                BranchIndex<usize>,
            >(
                model,
                scc,
                &sccs,
                &DominatedByRelation::empty(),
                &values,
                &mut subgame_construction_context,
            );

            solve_subgame_via_ovi(
                &sub_model.mdp,
                &sub_model.choice_exit_values,
                precision_per_scc,
                &mut subgame_values,
                &mut subgame_upper_bound,
            );

            for new_state in sub_model.mdp.states() {
                values[sub_model.to_old_state_index[new_state]] = subgame_values[new_state];
            }
        }
    }
    values
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

fn solve_subgame_via_ovi<NewCI: Index, NewBI: Index>(
    mdp: &Mdp<StateIndex<usize>, NewCI, NewBI>,
    choice_exit_values: &To1<NewCI, f64>,
    mut eps: f64,
    values: &mut To1<StateIndex<usize>, f64>,
    upper_bound: &mut To1<StateIndex<usize>, f64>,
) {
    for state in mdp.states() {
        values[state] = 0.0;
    }

    let initial_eps = eps;
    loop {
        subgame_value_iteration(mdp, choice_exit_values, eps, values);

        for state in mdp.states() {
            upper_bound[state] = match values[state] {
                0.0 => 0.0,
                v => (v + initial_eps).min(1.0),
            }
        }

        match verify_subgame_optimistic(mdp, choice_exit_values, 2.0 * eps, values, upper_bound) {
            OptimisticValueIterationResult::UpperBoundVerified => {
                for state in mdp.states() {
                    values[state] = 0.5 * (values[state] + upper_bound[state]);
                }
                break;
            }
            OptimisticValueIterationResult::UpperBoundRefuted { error } => {
                eps = error * 0.5;
            }
        }
    }
}

fn subgame_value_iteration<NewCI: Index, NewBI: Index>(
    mdp: &Mdp<StateIndex<usize>, NewCI, NewBI>,
    choice_exit_values: &To1<NewCI, f64>,
    eps: f64,
    values: &mut To1<StateIndex<usize>, f64>,
) {
    loop {
        let mut largest_change = 0.0;
        for state in mdp.states() {
            let mut best_value = 0.0;
            for choice in mdp.choices_of_state(state) {
                let mut value = choice_exit_values[choice];
                for branch in mdp.branches_of_choice(choice) {
                    value +=
                        mdp.branch_probability(branch) * values[mdp.branch_destination(branch)];
                }
                if value >= best_value {
                    best_value = value;
                }
            }

            let absolute_error = best_value - values[state];
            let relative_error = absolute_error / best_value;
            if relative_error > largest_change {
                largest_change = relative_error;
            }
            values[state] = best_value;
        }
        if largest_change < eps {
            break;
        }
    }
}

fn verify_subgame_optimistic<NewCI: Index, NewBI: Index>(
    mdp: &Mdp<StateIndex<usize>, NewCI, NewBI>,
    choice_exit_values: &To1<NewCI, f64>,
    eps: f64,
    values: &mut To1<StateIndex<usize>, f64>,
    upper_bound: &mut To1<StateIndex<usize>, f64>,
) -> OptimisticValueIterationResult {
    let verification_steps = (1.0 / eps).max(1.0) as usize;
    let mut error: f64 = 0.0;
    for _ in 0..verification_steps {
        let mut all_up = true;
        let mut all_down = true;
        error = 0.0;
        for state in mdp.states() {
            let mut new_lower_value = 0.0;
            let mut new_upper_value = 0.0;

            for choice in mdp.choices_of_state(state) {
                let mut lower_value = choice_exit_values[choice];
                let mut upper_value = choice_exit_values[choice];
                for branch in mdp.branches_of_choice(choice) {
                    lower_value +=
                        mdp.branch_probability(branch) * values[mdp.branch_destination(branch)];
                    upper_value += mdp.branch_probability(branch)
                        * upper_bound[mdp.branch_destination(branch)];
                }
                if lower_value >= new_lower_value {
                    new_lower_value = lower_value;
                }
                if upper_value >= new_upper_value {
                    new_upper_value = upper_value;
                }
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
