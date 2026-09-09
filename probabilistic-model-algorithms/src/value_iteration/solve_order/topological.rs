use crate::dominated_by::DominatedByRelation;
use crate::sccs::{SccDependencyIndex, SccEntryIndex, SccIndex, Sccs};
use crate::sub_model::{SubModel, SubModelConstructionContext};
use crate::value_iteration::non_determinism::NonDeterminism;
use crate::value_iteration::solve_order::{ModelSize, SolveOrder};
use crate::value_iteration::subgame_solver::SubGameSolver;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use probabilistic_models::{BranchIndex, ChoiceIndex, StateIndex};
use typed_index_collections::{Index, RawIndex, SemiboundedIndexRange, To1};

pub struct Topological {}

impl SolveOrder for Topological {
    fn find_and_solve_subgames<
        ND: NonDeterminism,
        Solver: SubGameSolver,
        M: ReadStateSpace + ReadPredecessors<StateIdx = M::StateIndex>,
    >(
        model: &M,
        s0: &To1<M::StateIndex, bool>,
        s1: &To1<M::StateIndex, bool>,
        dom_by: &DominatedByRelation<M::StateIndex>,
        eps: f64,
    ) -> To1<M::StateIndex, f64> {
        let mut values = create_value_vector(model.states(), &s1);

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> =
            Sccs::compute(model, Some((s0, s1)));
        let longest_chain = sccs
            .compute_dependencies::<SccDependencyIndex<usize>, _>(model)
            .longest_chain();
        let scc_eps = 2.0 * eps * (1.0 / longest_chain as f64);
        let max_size = sccs.max_size();
        let mut solver = Solver::create(max_size);
        let mut submodels = SubModelCollection::new();
        let mut submodel_context = SubModelConstructionContext::new(model);
        for scc in sccs.reverse_topological_ordering() {
            if let Some(state) = scc.as_singleton() {
                values[state] = if model.choices_of_state(state).len() == 0 {
                    0.0
                } else {
                    let mut best_value = ND::neutral_value();
                    for choice in model.choices_of_state(state) {
                        let choice_value = evaluate_choice(model, &values, state, choice);
                        if ND::is_better(best_value, choice_value) {
                            best_value = choice_value;
                        }
                    }
                    best_value
                }
            } else {
                let size = ModelSize::from_scc(model, scc);
                if size.fits_u8() {
                    let sm = &mut submodels.u8;
                    sm.rebuild_from_scc(model, scc, &dom_by, &values, &mut submodel_context);
                    let res = solver.solve::<ND, _, _, _>(&sm.mdp, &sm.choice_exit_values, scc_eps);
                    write_to_global_values(&mut values, sm, res);
                } else if size.fits_u16() {
                    let sm = &mut submodels.u16;
                    sm.rebuild_from_scc(model, scc, &dom_by, &values, &mut submodel_context);
                    let res = solver.solve::<ND, _, _, _>(&sm.mdp, &sm.choice_exit_values, scc_eps);
                    write_to_global_values(&mut values, sm, res);
                } else if size.fits_u32() {
                    let sm = &mut submodels.u32;
                    sm.rebuild_from_scc(model, scc, &dom_by, &values, &mut submodel_context);
                    let res = solver.solve::<ND, _, _, _>(&sm.mdp, &sm.choice_exit_values, scc_eps);
                    write_to_global_values(&mut values, sm, res);
                } else {
                    let sm = &mut submodels.usize;
                    sm.rebuild_from_scc(model, scc, &dom_by, &values, &mut submodel_context);
                    let res = solver.solve::<ND, _, _, _>(&sm.mdp, &sm.choice_exit_values, scc_eps);
                    write_to_global_values(&mut values, sm, res);
                };
            }
        }
        values
    }
}

fn write_to_global_values<OldSI: Index, SI: Index, CI: Index, BI: Index>(
    values: &mut To1<OldSI, f64>,
    sm: &SubModel<OldSI, SI, CI, BI>,
    res: &[f64],
) {
    for new_state in sm.mdp.states() {
        values[sm.to_old_state_index[new_state]] = res[new_state.raw().as_usize()];
    }
}

fn create_value_vector<StateIdx: Index>(
    states: SemiboundedIndexRange<StateIdx>,
    s1: &To1<StateIdx, bool>,
) -> To1<StateIdx, f64> {
    let mut values = To1::with_capacity(states.len());
    for state in states {
        values.add_checked(state, if s1[state] { 1.0 } else { 0.0 });
    }
    values
}

fn evaluate_choice<M: ReadStateSpace>(
    model: &M,
    values: &To1<M::StateIndex, f64>,
    state: M::StateIndex,
    choice: M::ChoiceIndex,
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

pub struct SubModelCollection<OldStateIdx: Index> {
    pub u8: SubModel<OldStateIdx, StateIndex<u8>, ChoiceIndex<u8>, BranchIndex<u8>>,
    pub u16: SubModel<OldStateIdx, StateIndex<u16>, ChoiceIndex<u16>, BranchIndex<u16>>,
    pub u32: SubModel<OldStateIdx, StateIndex<u32>, ChoiceIndex<u32>, BranchIndex<u32>>,
    pub usize: SubModel<OldStateIdx, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>,
}

impl<OldStateIdx: Index> SubModelCollection<OldStateIdx> {
    pub fn new() -> Self {
        Self {
            u8: SubModel::empty(),
            u16: SubModel::empty(),
            u32: SubModel::empty(),
            usize: SubModel::empty(),
        }
    }
}
