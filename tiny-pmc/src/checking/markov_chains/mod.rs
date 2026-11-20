use crate::checking::{CheckerError, CheckerOptions, CheckingResult};
use prism_model_builder::queries::ProcessedQuery;
use probabilistic_models::initial_states::SingleInitialState;
use probabilistic_models::traits::{
    ReadAtomicPropositionsMaybe, ReadPredecessors, ReadRewardsMaybe, ReadStateBranches,
};
use probabilistic_models::{AsRefComponent, Model};

pub fn check_dtmc<'a, B, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>(
    model: &'a Model<
        B,
        SingleInitialState<B::Ix>,
        ChLabel,
        BrLabel,
        Obs,
        APs,
        Rew,
        Ann,
        StateVals,
        Preds,
    >,
    query: ProcessedQuery<B::Ix>,
    options: &CheckerOptions,
) -> Result<CheckingResult<B::Ix>, CheckerError>
where
    B: ReadStateBranches,
    APs: AsRefComponent,
    APs::Output<'a>: ReadAtomicPropositionsMaybe<Ix = B::Ix>,
    Rew: AsRefComponent,
    Rew::Output<'a>: ReadRewardsMaybe<Ix = B::Ix>,
    Preds: ReadPredecessors<Ix = B::Ix>,
{
    let _ = (model, query, options);
    Err(CheckerError::NoSuitableAlgorithm)
}
