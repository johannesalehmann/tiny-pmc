use crate::sub_model::SubModelOrder;
use crate::value_iteration::solve_order::SubModelHook;
use crate::value_iteration::{EpsAllocationScheme, SccTimingOutput};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ValueIterationConfig {
    pub collapse_mecs: CollapseMecs,
    pub solve_order: SolveOrder,
    pub sub_model_order: SubModelOrder,
    pub eps: f64,
    // Can be used for benchmarking. Prints how long each sub_mdp (i.e. each SCC for topological VI)
    //  took to solve
    pub write_sub_mdp_timing: Option<SccTimingOutput>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CollapseMecs {
    WhenNecessary,
    WheneverPossible,
}

#[derive(Clone, Debug)]
pub enum SolveOrder {
    Topological {
        eps_allocation_scheme: EpsAllocationScheme,
        hook: Option<Rc<RefCell<dyn SubModelHook>>>,
    },
    Monolithic,
}
