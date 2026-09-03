mod buechi;
pub use buechi::{
    BuechiContext, buechi_winner_from_state, buechi_winner_from_state_raw, create_buechi_context,
    solve_buechi, solve_buechi_raw,
};

mod reachability;
pub use reachability::{
    ReachabilityContext, create_reachability_context, reachability_winner_from_state,
    reachability_winner_from_state_raw, solve_reachability, solve_reachability_raw,
};

mod safety;
pub use safety::{
    SafetyContext, create_safety_context, safety_winner_from_state, safety_winner_from_state_raw,
    solve_safety, solve_safety_raw,
};

// TODO: Perhaps enforce for the _raw functions that no game with owners is passed in? That way, it
//  is clear that these functions do not read from the game's ownership, instead
//  relying on the ownership stored in the context.

use crate::attractor::AttractorBuffer;
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{ReadAtomicPropositions, ReadOwners, ReadStateSpace};
use typed_index_collections::{Index, To1};

fn model_owners<
    M: ReadStateSpace + ReadOwners<StateIdx = <M as ReadStateSpace>::StateIdx, OwnerType = TwoPlayer>,
>(
    model: &M,
) -> To1<<M as ReadStateSpace>::StateIdx, TwoPlayer> {
    let mut owners = To1::with_capacity(model.states().len());
    for state in model.states() {
        owners.add_checked(state, model.state_owner(state));
    }
    owners
}

fn states_with_ap<
    M: ReadStateSpace + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    atomic_proposition: <M as ReadAtomicPropositions>::APIdx,
) -> Vec<<M as ReadStateSpace>::StateIdx> {
    model
        .states()
        .into_iter()
        .filter(|&state| model.is_atomic_proposition_set(state, atomic_proposition))
        .collect()
}

fn states_without_ap<
    M: ReadStateSpace + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    atomic_proposition: <M as ReadAtomicPropositions>::APIdx,
) -> Vec<<M as ReadStateSpace>::StateIdx> {
    model
        .states()
        .into_iter()
        .filter(|&state| !model.is_atomic_proposition_set(state, atomic_proposition))
        .collect()
}

fn reset_owner_counts<StateIdx: Index>(
    owners: &To1<StateIdx, TwoPlayer>,
    buffer: &mut AttractorBuffer<StateIdx>,
    reaching_player: TwoPlayer,
) {
    for (state, &owner) in owners.enumerate() {
        set_owner_count(buffer, state, owner, reaching_player);
    }
}

fn set_owner_count<StateIdx: Index>(
    buffer: &mut AttractorBuffer<StateIdx>,
    state: StateIdx,
    owner: TwoPlayer,
    reaching_player: TwoPlayer,
) {
    if owner == reaching_player {
        buffer.reset_reaching_player(state);
    } else {
        buffer.reset_avoiding_player(state);
    }
}
