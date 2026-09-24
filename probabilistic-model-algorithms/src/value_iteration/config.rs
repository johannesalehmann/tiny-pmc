use crate::value_iteration::{EpsAllocationScheme, SccTimingOutput};

pub struct ValueIterationConfig {
    pub collapse_mecs: CollapseMecs,
    pub solve_order: SolveOrder,
    pub eps: f64,
    // Can be used for benchmarking. Prints how long each sub_mdp (i.e. each SCC for topological VI)
    //  took to solve
    pub write_sub_mdp_timing: Option<SccTimingOutput>,
}

#[derive(Clone, Copy, Debug)]
pub enum CollapseMecs {
    WhenNecessary,
    WheneverPossible,
}

#[derive(Clone, Debug)]
pub enum SolveOrder {
    Monolithic,
    Topological {
        eps_allocation_scheme: EpsAllocationScheme,
    },
}
