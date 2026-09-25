use clap::{Args, Parser, ValueEnum};
use probabilistic_models::SuccessorOrder;
use std::path::PathBuf;
use tiny_pmc::checking::{
    CheckerOptions, CollapseMecs, EpsAllocationScheme, SccTimingOutput, SolveOrder, SubModelOrder,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[arg(short, long)]
    pub model: String,
    #[arg(short, long)]
    pub property: String,
    #[arg(short, long, default_value_t = String::new())]
    pub constants: String,
    #[arg(
        long,
        value_enum,
        num_args = 0..=1,
        default_missing_value = "preserve",
        value_name = "SUCCESSOR_ORDER"
    )]
    pub dfs: Option<DfsArg>,
    #[command(flatten)]
    pub value_iteration: ValueIterationArguments,
}

#[derive(Args)]
#[command(next_help_heading = "Value iteration")]
pub struct ValueIterationArguments {
    #[arg(long, default_value_t = 1e-6)]
    pub eps: f64,
    #[arg(long)]
    pub unsound: bool,
    #[arg(long, value_enum, default_value_t = SolveOrderArg::Topological)]
    pub solve_order: SolveOrderArg,
    #[arg(long = "topo.eps-allocation", value_enum)]
    pub eps_allocation: Option<EpsAllocationArg>,
    #[arg(long = "topo.scc-timings", value_name = "FILE", num_args = 0..=1)]
    pub scc_timings: Option<Option<PathBuf>>,
    #[arg(long, value_enum, default_value_t = CollapseMecsArg::WhenNecessary)]
    pub collapse_mecs: CollapseMecsArg,
    #[arg(long, value_enum, default_value_t = StateOrderArg::BackToFront)]
    pub state_order: StateOrderArg,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum DfsArg {
    Preserve,
    ByProbability,
}

impl From<DfsArg> for SuccessorOrder {
    fn from(value: DfsArg) -> Self {
        match value {
            DfsArg::Preserve => SuccessorOrder::Preserve,
            DfsArg::ByProbability => SuccessorOrder::HighestProbabilityFirst,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum CollapseMecsArg {
    WhenNecessary,
    WheneverPossible,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum SolveOrderArg {
    Monolithic,
    Topological,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum StateOrderArg {
    BackToFront,
    FrontToBack,
    AttractorStyle,
    Legacy,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum EpsAllocationArg {
    Uniform,
    GlobalEpsForEach,
}

impl ValueIterationArguments {
    pub fn to_checker_options(&self) -> Result<CheckerOptions, String> {
        let solve_order = match self.solve_order {
            SolveOrderArg::Monolithic => {
                if self.eps_allocation.is_some() || self.scc_timings.is_some() {
                    return Err(
                        "The options `--topo.*` require `--solve-order topological`".to_string()
                    );
                }
                SolveOrder::Monolithic
            }
            SolveOrderArg::Topological => SolveOrder::Topological {
                eps_allocation_scheme: match self.eps_allocation {
                    None | Some(EpsAllocationArg::Uniform) => EpsAllocationScheme::Uniform,
                    Some(EpsAllocationArg::GlobalEpsForEach) => {
                        EpsAllocationScheme::GlobalEpsForEach
                    }
                },
            },
        };
        let write_scc_timing = match &self.scc_timings {
            None => None,
            Some(None) => Some(SccTimingOutput::Stdout),
            Some(Some(path)) => Some(SccTimingOutput::File(path.clone())),
        };
        Ok(CheckerOptions {
            eps: self.eps,
            sound: !self.unsound,
            solve_order,
            collapse_mecs: match self.collapse_mecs {
                CollapseMecsArg::WhenNecessary => CollapseMecs::WhenNecessary,
                CollapseMecsArg::WheneverPossible => CollapseMecs::WheneverPossible,
            },
            sub_model_order: match self.state_order {
                StateOrderArg::BackToFront => SubModelOrder::BackToFront,
                StateOrderArg::FrontToBack => SubModelOrder::FrontToBack,
                StateOrderArg::AttractorStyle => SubModelOrder::AttractorStyle,
                StateOrderArg::Legacy => SubModelOrder::Legacy,
            },
            write_scc_timing,
        })
    }
}
