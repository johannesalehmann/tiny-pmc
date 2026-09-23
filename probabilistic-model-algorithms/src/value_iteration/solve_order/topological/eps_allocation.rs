use crate::sccs::{SccDependencyIndex, Sccs};
use crate::value_iteration::solve_order::ModelSize;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use typed_index_collections::Index;

#[derive(Clone, Copy, Debug)]
pub enum EpsAllocationScheme {
    Uniform,
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
        let longest_chain = sccs
            .compute_dependencies::<SccDependencyIndex<usize>, _>(model)
            .longest_chain();
        // TODO: This way of distributing eps is incorrect, it should be multiplicative
        let scc_eps = 2.0 * global_eps * (1.0 / longest_chain as f64);
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
