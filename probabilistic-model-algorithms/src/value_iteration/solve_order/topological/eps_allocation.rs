use crate::sccs::SccChainWeight;
use crate::value_iteration::solve_order::ModelSize;
use typed_index_collections::Index;

#[derive(Clone, Copy, Debug)]
pub enum EpsAllocationScheme {
    Uniform,
    Proportional,
    UniformUnsound,
    GlobalEpsForEach,
}

pub trait EpsAllocation<SccIndex: Index> {
    const CHAIN_WEIGHT: Option<SccChainWeight>;
    fn create(global_eps: f64, longest_chain: Option<usize>) -> Self;
    fn eps(&self, scc_index: SccIndex, size: &ModelSize) -> f64;
}

pub struct UniformUnsoundEpsAllocation {
    scc_eps: f64,
}

impl<SccIndex: Index> EpsAllocation<SccIndex> for UniformUnsoundEpsAllocation {
    const CHAIN_WEIGHT: Option<SccChainWeight> = Some(SccChainWeight::CountAllScc);

    fn create(global_eps: f64, longest_chain: Option<usize>) -> Self {
        // This way of distributing eps is unsound and can lead to results that are slightly too
        // large. We keep it because, in practice, results are usually very close. In most cases
        // `UniformEpsAllocation` gives almost the same result.
        let scc_eps = global_eps / longest_chain.unwrap() as f64;
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
    const CHAIN_WEIGHT: Option<SccChainWeight> = None;

    fn create(global_eps: f64, _longest_chain: Option<usize>) -> Self {
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
    const CHAIN_WEIGHT: Option<SccChainWeight> = Some(SccChainWeight::CountNonSingletonSccs);

    fn create(global_eps: f64, longest_chain: Option<usize>) -> Self {
        Self {
            scc_eps: share_of_eps(global_eps, 1, longest_chain.unwrap()),
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
    const CHAIN_WEIGHT: Option<SccChainWeight> = Some(SccChainWeight::SizeOfNonSingletonSccs);

    fn create(global_eps: f64, longest_chain: Option<usize>) -> Self {
        Self {
            global_eps,
            heaviest_non_singleton_chain: longest_chain.unwrap(),
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
