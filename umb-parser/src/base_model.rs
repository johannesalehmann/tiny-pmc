use crate::archive::UmbArchive;
use crate::index::TransitionSystem;
use crate::{BI, CI, SI, UmbError, to1};
use probabilistic_models::base_model::{
    BranchGraph, ChoiceBranchGraph, ChoiceGraph, Dtmc, Mdp, NonstochasticGame, StochasticGame,
    TransitionSystem as Ts, TurnBased,
};
use probabilistic_models::typed_index_collections::NamedTo1;
use probabilistic_models::{DefaultIx, Index, PlayerIndex};
// TODO: To1 defaults are currently not properly supported. For example, in a game where every state
//  is controlled by player 1, it is ok to omit the state-to-player file. Instead, the code below
//  produces an error in that case.

pub fn parse_mdp(archive: &mut UmbArchive, sizes: &TransitionSystem) -> Result<Mdp, UmbError> {
    Ok(Mdp::with_graph(parse_choice_branch_graph(archive, sizes)?))
}

pub fn parse_dtmc(archive: &mut UmbArchive, sizes: &TransitionSystem) -> Result<Dtmc, UmbError> {
    Ok(Dtmc::with_graph(parse_branch_graph(archive, sizes)?))
}

pub fn parse_ts(archive: &mut UmbArchive, sizes: &TransitionSystem) -> Result<Ts, UmbError> {
    Ok(Ts::with_graph(parse_choice_graph(archive, sizes)?))
}

pub fn parse_nonstochastic_game(
    archive: &mut UmbArchive,
    sizes: &TransitionSystem,
) -> Result<NonstochasticGame, UmbError> {
    Ok(NonstochasticGame::with_graph_and_players(
        parse_choice_graph(archive, sizes)?,
        parse_players(archive, sizes)?,
    ))
}

pub fn parse_smg(
    archive: &mut UmbArchive,
    sizes: &TransitionSystem,
) -> Result<StochasticGame, UmbError> {
    Ok(StochasticGame::with_graph_and_players(
        parse_choice_branch_graph(archive, sizes)?,
        parse_players(archive, sizes)?,
    ))
}

fn parse_choice_branch_graph(
    archive: &mut UmbArchive,
    sizes: &TransitionSystem,
) -> Result<ChoiceBranchGraph<DefaultIx>, UmbError> {
    let state_to_choice = archive.csr_or_identity("state-to-choices.bin", sizes.num_states)?;
    let choice_to_branch = archive.csr_or_identity("choice-to-branches.bin", sizes.num_choices)?;
    let branch_destinations =
        to1::parse_to1::<BI, u64, _>(archive.read_required_file("branch-to-target.bin")?)?
            .map(|i| SI::from_raw(*i as usize));
    let branch_probabilities =
        to1::parse_to1::<BI, f64, _>(archive.read_required_file("branch-to-probability.bin")?)?;
    let choice_branch_graph = ChoiceBranchGraph {
        state_to_choice,
        choice_to_branch,
        branch_probabilities,
        branch_destinations,
    };
    Ok(choice_branch_graph)
}

fn parse_choice_graph(
    archive: &mut UmbArchive,
    sizes: &TransitionSystem,
) -> Result<ChoiceGraph<DefaultIx>, UmbError> {
    let state_to_choice = archive.csr_or_identity("state-to-choices.bin", sizes.num_states)?;
    let choice_destinations =
        to1::parse_to1::<CI, u64, _>(archive.read_required_file("branch-to-target.bin")?)?
            .map(|i| SI::from_raw(*i as usize));
    let choice_graph = ChoiceGraph {
        state_to_choice,
        choice_destinations,
    };
    Ok(choice_graph)
}
fn parse_branch_graph(
    archive: &mut UmbArchive,
    sizes: &TransitionSystem,
) -> Result<BranchGraph<DefaultIx>, UmbError> {
    let state_to_branch = archive.csr_or_identity("choice-to-branches.bin", sizes.num_choices)?;
    let branch_destinations =
        to1::parse_to1::<BI, u64, _>(archive.read_required_file("branch-to-target.bin")?)?
            .map(|i| SI::from_raw(*i as usize));
    let branch_probabilities =
        to1::parse_to1::<BI, f64, _>(archive.read_required_file("branch-to-probability.bin")?)?;
    let branch_graph = BranchGraph {
        state_to_branch,
        branch_probabilities,
        branch_destinations,
    };
    Ok(branch_graph)
}

fn parse_players(
    archive: &mut UmbArchive,
    sizes: &TransitionSystem,
) -> Result<TurnBased<DefaultIx>, UmbError> {
    let state_to_owner =
        to1::parse_to1::<SI, u32, _>(archive.read_required_file("state-to-player.bin")?)?
            .map(|i| PlayerIndex::from_raw(i.strict_sub(1))); // Subtract 1 because player indices are 1-based in UMB.
    match &sizes.player_names {
        None => panic!("missing player names"),
        Some(names) => {
            let players = NamedTo1::with_entries(names.iter().map(|n| (n.clone(), ())));
            Ok(TurnBased {
                state_to_owner,
                players,
            })
        }
    }
}
