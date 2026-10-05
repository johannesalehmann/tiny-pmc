mod eps_allocation;

pub use eps_allocation::{EpsAllocationScheme, GlobalEpsForEachScc, UniformEpsAllocation};
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

mod scc_timings;
use crate::mecs::Mecs;
use crate::sccs::{SccEntryIndex, SccIndex, Sccs};
use crate::sub_model::{
    Attractor, IndexBased, IndexOrderDirection, Legacy, RewardsSource, StateOrdering, SubModel,
    SubModelConstructionContext, SubModelOrder,
};
use crate::value_iteration::non_determinism::NonDeterminismResolver;
use crate::value_iteration::precomputed_states::PrecomputedStates;
use crate::value_iteration::solve_order::topological::eps_allocation::EpsAllocation;
use crate::value_iteration::solve_order::{ModelSize, OrderedSolver};
use crate::value_iteration::sub_model_solver::SubModelSolver;
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use probabilistic_models::{BranchIndex, ChoiceIndex, StateIndex};
pub use scc_timings::{SccTimingOutput, SccTimings, TopoTiming};
use typed_index_collections::{Index, RawIndex, SemiboundedIndexRange, To1};

pub struct Topological {
    timing: Option<SccTimingOutput>,
    eps_allocation_scheme: EpsAllocationScheme,
    sub_model_order: SubModelOrder,
    hook: Option<Rc<RefCell<dyn SubModelHook>>>,
}

impl Topological {
    pub fn new(
        timing: Option<SccTimingOutput>,
        eps_allocation_scheme: EpsAllocationScheme,
        sub_model_order: SubModelOrder,
        hook: Option<Rc<RefCell<dyn SubModelHook>>>,
    ) -> Self {
        Self {
            timing,
            eps_allocation_scheme,
            sub_model_order,
            hook,
        }
    }
}

impl OrderedSolver for Topological {
    fn find_and_solve_submodels<
        ND: NonDeterminismResolver,
        Solver: SubModelSolver,
        M: ReadStateSpace
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
        P: PrecomputedStates<StateIdx = M::StateIndex>,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
    >(
        self,
        model: &M,
        precomputed_states: &P,
        rew: Rew,
        mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
        eps: f64,
    ) -> To1<M::StateIndex, f64> {
        match self.timing.clone() {
            None => self.find_and_solve_submodels_with_timing::<ND, Solver, M, P, Rew, _>(
                model,
                precomputed_states,
                rew,
                mecs,
                eps,
                (),
            ),
            Some(output) => self.find_and_solve_submodels_with_timing::<ND, Solver, M, P, Rew, _>(
                model,
                precomputed_states,
                rew,
                mecs,
                eps,
                SccTimings::new(output),
            ),
        }
    }
}

impl Topological {
    fn find_and_solve_submodels_with_timing<
        ND: NonDeterminismResolver,
        Solver: SubModelSolver,
        M: ReadStateSpace
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
        P: PrecomputedStates<StateIdx = M::StateIndex>,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
        Timing: TopoTiming,
    >(
        self,
        model: &M,
        precomputed_states: &P,
        rew: Rew,
        mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
        eps: f64,
        timing: Timing,
    ) -> To1<M::StateIndex, f64> {
        match self.eps_allocation_scheme {
            EpsAllocationScheme::Uniform => {
                self.find_and_solve_submodels_with_timing_and_eps_allocation::<ND, Solver, M, P, Rew, Timing, UniformEpsAllocation>(model, precomputed_states, rew, mecs, eps, timing)
            }
            EpsAllocationScheme::GlobalEpsForEach => {
                self.find_and_solve_submodels_with_timing_and_eps_allocation::<ND, Solver, M, P, Rew, Timing, GlobalEpsForEachScc>(model, precomputed_states, rew, mecs, eps, timing)
            }
        }
    }
    fn find_and_solve_submodels_with_timing_and_eps_allocation<
        ND: NonDeterminismResolver,
        Solver: SubModelSolver,
        M: ReadStateSpace
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
        P: PrecomputedStates<StateIdx = M::StateIndex>,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
        Timing: TopoTiming,
        EA: EpsAllocation<SccIndex<usize>>,
    >(
        self,
        model: &M,
        precomputed_states: &P,
        rew: Rew,
        mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
        eps: f64,
        timings: Timing,
    ) -> To1<M::StateIndex, f64> {
        match self.sub_model_order {
            SubModelOrder::BackToFront => self
                .find_and_solve_submodels_with_ordering::<ND, Solver, M, P, Rew, Timing, EA, _>(
                    model,
                    precomputed_states,
                    rew,
                    mecs,
                    eps,
                    timings,
                    IndexBased::new(IndexOrderDirection::BackToFront),
                ),
            SubModelOrder::FrontToBack => self
                .find_and_solve_submodels_with_ordering::<ND, Solver, M, P, Rew, Timing, EA, _>(
                    model,
                    precomputed_states,
                    rew,
                    mecs,
                    eps,
                    timings,
                    IndexBased::new(IndexOrderDirection::FrontToBack),
                ),
            SubModelOrder::Attractor(choice_mode) => self
                .find_and_solve_submodels_with_ordering::<ND, Solver, M, P, Rew, Timing, EA, _>(
                    model,
                    precomputed_states,
                    rew,
                    mecs,
                    eps,
                    timings,
                    Attractor::new(choice_mode),
                ),
            SubModelOrder::Legacy => self
                .find_and_solve_submodels_with_ordering::<ND, Solver, M, P, Rew, Timing, EA, _>(
                    model,
                    precomputed_states,
                    rew,
                    mecs,
                    eps,
                    timings,
                    Legacy,
                ),
        }
    }

    fn find_and_solve_submodels_with_ordering<
        ND: NonDeterminismResolver,
        Solver: SubModelSolver,
        M: ReadStateSpace
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
        P: PrecomputedStates<StateIdx = M::StateIndex>,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
        Timing: TopoTiming,
        EA: EpsAllocation<SccIndex<usize>>,
        O: StateOrdering,
    >(
        mut self,
        model: &M,
        precomputed_states: &P,
        rew: Rew,
        mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
        eps: f64,
        mut timings: Timing,
        ordering: O,
    ) -> To1<M::StateIndex, f64> {
        let mut values = create_value_vector(model.states(), precomputed_states);
        let max = precomputed_states.max_value();

        let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> =
            Sccs::compute(model, precomputed_states);
        let eps_allocation = EA::create(eps, model, &sccs);
        let max_size = sccs.max_size();
        let mut solver = Solver::create(max_size);
        let mut submodels = SubModelCollection::new();
        let mut submodel_context = SubModelConstructionContext::new(model, &ordering);
        for scc in sccs.reverse_topological_ordering() {
            if let Some(state) = scc.as_singleton() {
                values[state] = if model.choices_of_state(state).len() == 0 {
                    0.0
                } else {
                    let mut best_value = ND::neutral_value();
                    for choice in model.choices_of_state(state) {
                        let choice_value =
                            evaluate_choice::<ND, _, _>(model, &values, &rew, state, choice);
                        if ND::is_better(best_value, choice_value) {
                            best_value = choice_value;
                        }
                    }
                    best_value
                }
            } else {
                let size = ModelSize::from_scc(model, scc);
                let scc_eps = eps_allocation.eps(scc.get_index(), &size);
                let timing_entry = timings.start_entry(&size);
                if size.fits_u8() {
                    let sm = &mut submodels.u8;
                    sm.rebuild_from_scc(
                        model,
                        scc,
                        mecs,
                        &values,
                        &rew,
                        &ordering,
                        &mut submodel_context,
                    );
                    if let Some(hook) = &mut self.hook {
                        hook.borrow_mut()
                            .handle_sub_model_u8(&sm.mdp, &sm.choice_exit_values);
                    }
                    let res =
                        solver.solve::<ND, _, _, _>(&sm.mdp, &sm.choice_exit_values, scc_eps, max);
                    write_to_global_values(&mut values, sm, res);
                } else if size.fits_u16() {
                    let sm = &mut submodels.u16;
                    sm.rebuild_from_scc(
                        model,
                        scc,
                        mecs,
                        &values,
                        &rew,
                        &ordering,
                        &mut submodel_context,
                    );
                    if let Some(hook) = &mut self.hook {
                        hook.borrow_mut()
                            .handle_sub_model_u16(&sm.mdp, &sm.choice_exit_values);
                    }
                    let res =
                        solver.solve::<ND, _, _, _>(&sm.mdp, &sm.choice_exit_values, scc_eps, max);

                    write_to_global_values(&mut values, sm, res);
                } else if size.fits_u32() {
                    let sm = &mut submodels.u32;
                    sm.rebuild_from_scc(
                        model,
                        scc,
                        mecs,
                        &values,
                        &rew,
                        &ordering,
                        &mut submodel_context,
                    );
                    if let Some(hook) = &mut self.hook {
                        hook.borrow_mut()
                            .handle_sub_model_u32(&sm.mdp, &sm.choice_exit_values);
                    }
                    let res =
                        solver.solve::<ND, _, _, _>(&sm.mdp, &sm.choice_exit_values, scc_eps, max);
                    write_to_global_values(&mut values, sm, res);
                } else {
                    let sm = &mut submodels.usize;
                    sm.rebuild_from_scc(
                        model,
                        scc,
                        mecs,
                        &values,
                        &rew,
                        &ordering,
                        &mut submodel_context,
                    );
                    if let Some(hook) = &mut self.hook {
                        hook.borrow_mut()
                            .handle_sub_model_usize(&sm.mdp, &sm.choice_exit_values);
                    }
                    let res =
                        solver.solve::<ND, _, _, _>(&sm.mdp, &sm.choice_exit_values, scc_eps, max);
                    write_to_global_values(&mut values, sm, res);
                };
                for state in scc.states() {
                    if let Some(representative) = mecs.representative(state) {
                        values[state] = values[representative];
                    }
                }
                timings.finish_entry(timing_entry);
            }
        }

        timings.write_topo_timings();

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
    precomputed_states: &impl PrecomputedStates<StateIdx = StateIdx>,
) -> To1<StateIdx, f64> {
    let mut values = To1::with_capacity(states.len());
    for state in states {
        values.add_checked(state, precomputed_states.initial_value(state));
    }
    values
}

fn evaluate_choice<
    ND: NonDeterminismResolver,
    M: ReadStateSpace,
    Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
>(
    model: &M,
    values: &To1<M::StateIndex, f64>,
    rew: &Rew,
    state: M::StateIndex,
    choice: M::ChoiceIndex,
) -> f64 {
    let mut to_self = 0.0;
    let mut exit_value = 0.0;
    if rew.has_state_rewards() {
        exit_value += rew.state_reward(state);
    }
    if rew.has_choice_rewards() {
        exit_value += rew.choice_reward(choice);
    }
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
        ND::neutral_value()
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

// Trait that can be used to exfiltrate arbitrary information about the sub-model. In particular,
// this is currently used by a benchmark that needs access to each individual SCC of a model for
// further analysis
// For other tasks, it might be required to pass additional values into the functions, e.g.
// the values computed by VI, how long it took to solve the model, etc.
pub trait SubModelHook: Debug {
    fn handle_sub_model_u8(
        &mut self,
        sub_model: &Mdp<StateIndex<u8>, ChoiceIndex<u8>, BranchIndex<u8>>,
        choice_exit_values: &To1<ChoiceIndex<u8>, f64>,
    );
    fn handle_sub_model_u16(
        &mut self,
        sub_model: &Mdp<StateIndex<u16>, ChoiceIndex<u16>, BranchIndex<u16>>,
        choice_exit_values: &To1<ChoiceIndex<u16>, f64>,
    );
    fn handle_sub_model_u32(
        &mut self,
        sub_model: &Mdp<StateIndex<u32>, ChoiceIndex<u32>, BranchIndex<u32>>,
        choice_exit_values: &To1<ChoiceIndex<u32>, f64>,
    );
    fn handle_sub_model_usize(
        &mut self,
        sub_model: &Mdp<StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>,
        choice_exit_values: &To1<ChoiceIndex<usize>, f64>,
    );
}
