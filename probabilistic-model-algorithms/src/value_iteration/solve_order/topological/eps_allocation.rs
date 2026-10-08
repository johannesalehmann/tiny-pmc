use crate::sccs::{SccDependencyIndex, Sccs};
use crate::value_iteration::solve_order::ModelSize;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use typed_index_collections::Index;

#[derive(Clone, Copy, Debug)]
pub enum EpsAllocationScheme {
    Uniform,
    Proportional,
    UniformUnsound,
    GlobalEpsForEach,
}

pub trait EpsAllocation<SccIndex: Index> {
    fn create<
        ScEI: Index,
        SI: Index,
        M: ReadStateSpace<StateIndex = SI> + ReadPredecessors<StateIdx = SI>,
    >(
        global_eps: f64,
        model: &M,
        sccs: &Sccs<SccIndex, ScEI, SI>,
    ) -> Self;
    fn eps(&self, scc_index: SccIndex, size: &ModelSize) -> f64;
}

pub struct UniformUnsoundEpsAllocation {
    scc_eps: f64,
}

impl<SccIndex: Index> EpsAllocation<SccIndex> for UniformUnsoundEpsAllocation {
    fn create<
        ScEI: Index,
        SI: Index,
        M: ReadStateSpace<StateIndex = SI> + ReadPredecessors<StateIdx = SI>,
    >(
        global_eps: f64,
        model: &M,
        sccs: &Sccs<SccIndex, ScEI, SI>,
    ) -> Self {
        // This way of distributing eps is unsound and can lead to results that are slightly too
        // large. We keep it because, in practice, results are usually very close. In most cases
        // `UniformEpsAllocation` gives almost the same result.
        let longest_chain = sccs
            .compute_dependencies::<SccDependencyIndex<usize>, _, _>(model, &())
            .longest_chain();
        let scc_eps = global_eps / longest_chain as f64;
        Self { scc_eps }
    }

    fn eps(&self, _scc_index: SccIndex, _size: &ModelSize) -> f64 {
        self.scc_eps
    }
}

pub struct GlobalEpsForEachScc {
    global_eps: f64,
}

impl<SccIndex: Index> EpsAllocation<SccIndex> for GlobalEpsForEachScc {
    fn create<
        ScEI: Index,
        SI: Index,
        M: ReadStateSpace<StateIndex = SI> + ReadPredecessors<StateIdx = SI>,
    >(
        global_eps: f64,
        _model: &M,
        _sccs: &Sccs<SccIndex, ScEI, SI>,
    ) -> Self {
        Self { global_eps }
    }

    fn eps(&self, _scc_index: SccIndex, _size: &ModelSize) -> f64 {
        self.global_eps
    }
}

// Computes (1 + global_eps)^(weight / total_weight) - 1. Using `ln_1p` and `exp_m1` avoids the
// floating-point issues that the naive formula suffers from for small eps.
fn share_of_eps(global_eps: f64, weight: usize, total_weight: usize) -> f64 {
    (global_eps.ln_1p() * weight as f64 / total_weight as f64).exp_m1()
}

pub struct UniformEpsAllocation {
    scc_eps: f64,
}

impl<SccIndex: Index> EpsAllocation<SccIndex> for UniformEpsAllocation {
    fn create<
        ScEI: Index,
        SI: Index,
        M: ReadStateSpace<StateIndex = SI> + ReadPredecessors<StateIdx = SI>,
    >(
        global_eps: f64,
        model: &M,
        sccs: &Sccs<SccIndex, ScEI, SI>,
    ) -> Self {
        let longest_non_singleton_chain = sccs
            .compute_dependencies::<SccDependencyIndex<usize>, _, _>(model, &())
            .longest_non_singleton_chain(sccs);
        Self {
            scc_eps: share_of_eps(global_eps, 1, longest_non_singleton_chain),
        }
    }

    fn eps(&self, _scc_index: SccIndex, _size: &ModelSize) -> f64 {
        self.scc_eps
    }
}

pub struct ProportionalEpsAllocation {
    global_eps: f64,
    heaviest_non_singleton_chain: usize,
}

impl<SccIndex: Index> EpsAllocation<SccIndex> for ProportionalEpsAllocation {
    fn create<
        ScEI: Index,
        SI: Index,
        M: ReadStateSpace<StateIndex = SI> + ReadPredecessors<StateIdx = SI>,
    >(
        global_eps: f64,
        model: &M,
        sccs: &Sccs<SccIndex, ScEI, SI>,
    ) -> Self {
        let heaviest_non_singleton_chain = sccs
            .compute_dependencies::<SccDependencyIndex<usize>, _, _>(model, &())
            .heaviest_non_singleton_chain(sccs, |scc| {
                ModelSize::from_scc(model, sccs.scc(scc)).weight()
            });
        Self {
            global_eps,
            heaviest_non_singleton_chain,
        }
    }

    fn eps(&self, _scc_index: SccIndex, size: &ModelSize) -> f64 {
        share_of_eps(
            self.global_eps,
            size.weight(),
            self.heaviest_non_singleton_chain,
        )
    }
}
