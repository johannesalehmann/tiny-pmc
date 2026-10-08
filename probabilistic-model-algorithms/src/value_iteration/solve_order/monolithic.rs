use super::ModelSize;
use super::topological::{create_value_vector, write_to_global_values};
use crate::mecs::Mecs;
use crate::sccs::{Scc, SccEntryIndex, SccIndex, Sccs};
use crate::sub_model::{
    Attractor, IndexBased, IndexOrderDirection, Legacy, RewardsSource, StateOrdering, SubModel,
    SubModelOrder,
};
use crate::value_iteration::non_determinism::NonDeterminismResolver;
use crate::value_iteration::precomputed_states::PrecomputedStates;
use crate::value_iteration::sub_model_solver::SubModelSolver;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use probabilistic_models::{BranchIndex, ChoiceIndex, StateIndex};
use typed_index_collections::{Index, To1};

pub struct Monolithic {
    sub_model_order: SubModelOrder,
}

impl Monolithic {
    pub fn new(sub_model_order: SubModelOrder) -> Self {
        Self { sub_model_order }
    }
}

impl super::OrderedSolver for Monolithic {
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
        match self.sub_model_order {
            SubModelOrder::BackToFront => solve_with_ordering::<ND, Solver, _, _, _, _>(
                model,
                precomputed_states,
                rew,
                mecs,
                eps,
                IndexBased::new(IndexOrderDirection::BackToFront),
            ),
            SubModelOrder::FrontToBack => solve_with_ordering::<ND, Solver, _, _, _, _>(
                model,
                precomputed_states,
                rew,
                mecs,
                eps,
                IndexBased::new(IndexOrderDirection::FrontToBack),
            ),
            SubModelOrder::Attractor(choice_mode) => solve_with_ordering::<ND, Solver, _, _, _, _>(
                model,
                precomputed_states,
                rew,
                mecs,
                eps,
                Attractor::new(choice_mode),
            ),
            SubModelOrder::Legacy => solve_with_ordering::<ND, Solver, _, _, _, _>(
                model,
                precomputed_states,
                rew,
                mecs,
                eps,
                Legacy,
            ),
        }
    }
}

fn solve_with_ordering<
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
    O: StateOrdering,
>(
    model: &M,
    precomputed_states: &P,
    rew: Rew,
    mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
    eps: f64,
    ordering: O,
) -> To1<M::StateIndex, f64> {
    let mut values = create_value_vector(model.states(), precomputed_states);

    // All maybe states are put into a single (not necessarily strongly connected) pseudo-SCC, so
    // that the sub-model construction used by topological VI can be reused.
    // TODO: It might be more efficient to build the submodel directly from precomputed_states
    //  instead of taking the detour via the pseudo-SCC.
    let maybe_states = model
        .states()
        .into_iter()
        .filter(|&state| precomputed_states.is_maybe_state(state));
    let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> =
        Sccs::new([maybe_states], model.states().len());
    let scc = sccs.scc(SccIndex::from_raw(0));
    let size = ModelSize::from_scc(model, scc);
    if size.states == 0 {
        return values;
    }

    let max = precomputed_states.max_value();
    if size.fits_u8() {
        solve::<ND, Solver, _, _, _, StateIndex<u8>, ChoiceIndex<u8>, BranchIndex<u8>>(
            model,
            scc,
            mecs,
            &mut values,
            &rew,
            &ordering,
            eps,
            max,
        );
    } else if size.fits_u16() {
        solve::<ND, Solver, _, _, _, StateIndex<u16>, ChoiceIndex<u16>, BranchIndex<u16>>(
            model,
            scc,
            mecs,
            &mut values,
            &rew,
            &ordering,
            eps,
            max,
        );
    } else if size.fits_u32() {
        solve::<ND, Solver, _, _, _, StateIndex<u32>, ChoiceIndex<u32>, BranchIndex<u32>>(
            model,
            scc,
            mecs,
            &mut values,
            &rew,
            &ordering,
            eps,
            max,
        );
    } else {
        solve::<ND, Solver, _, _, _, StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>(
            model,
            scc,
            mecs,
            &mut values,
            &rew,
            &ordering,
            eps,
            max,
        );
    }

    for state in model.states() {
        if let Some(representative) = mecs.representative(state) {
            values[state] = values[representative];
        }
    }
    values
}

fn solve<
    ND: NonDeterminismResolver,
    Solver: SubModelSolver,
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        >,
    Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
    O: StateOrdering,
    SI: Index,
    CI: Index,
    BI: Index,
>(
    model: &M,
    scc: Scc<'_, SccIndex<usize>, SccEntryIndex<usize>, M::StateIndex>,
    mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
    values: &mut To1<M::StateIndex, f64>,
    rew: &Rew,
    ordering: &O,
    eps: f64,
    max_value: f64,
) {
    let sm: SubModel<M::StateIndex, SI, CI, BI> =
        SubModel::from_scc(model, scc, mecs, values, rew, ordering);
    let mut solver = Solver::create(scc.size());
    let res = solver.solve::<ND, _, _, _>(&sm.mdp, &sm.choice_exit_values, eps, max_value);
    write_to_global_values(values, &sm, res);
}
