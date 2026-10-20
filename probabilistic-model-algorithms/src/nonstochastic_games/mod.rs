mod buechi;
pub use buechi::{
    BuechiContext, buechi_winner_from_state, buechi_winner_from_state_raw, create_buechi_context,
    solve_buechi, solve_buechi_raw,
};
use probabilistic_models::AtomicPropositionIndex;
use probabilistic_models::{Ix, StateIndex};

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
use probabilistic_models::base_model::{ReadOwners, TwoPlayer};
use probabilistic_models::traits::{ReadAtomicPropositions, ReadStateSpace};
use typed_index_collections::To1;

fn model_owners<M: ReadStateSpace + ReadOwners<OwnerType = TwoPlayer>>(
    model: &M,
) -> To1<StateIndex<M::Ix>, TwoPlayer> {
    let mut owners = To1::with_capacity(model.states().len());
    for state in model.states() {
        owners.add_checked(state, model.state_owner(state));
    }
    owners
}

fn states_with_ap<M: ReadStateSpace + ReadAtomicPropositions>(
    model: &M,
    atomic_proposition: AtomicPropositionIndex<M::Ix>,
) -> Vec<StateIndex<M::Ix>> {
    model
        .states()
        .into_iter()
        .filter(|&state| model.is_atomic_proposition_set(state, atomic_proposition))
        .collect()
}

fn states_without_ap<M: ReadStateSpace + ReadAtomicPropositions>(
    model: &M,
    atomic_proposition: AtomicPropositionIndex<M::Ix>,
) -> Vec<StateIndex<M::Ix>> {
    model
        .states()
        .into_iter()
        .filter(|&state| !model.is_atomic_proposition_set(state, atomic_proposition))
        .collect()
}

fn reset_owner_counts<I: Ix>(
    owners: &To1<StateIndex<I>, TwoPlayer>,
    buffer: &mut AttractorBuffer<I>,
    reaching_player: TwoPlayer,
) {
    for (state, &owner) in owners.enumerate() {
        set_owner_count(buffer, state, owner, reaching_player);
    }
}

fn set_owner_count<I: Ix>(
    buffer: &mut AttractorBuffer<I>,
    state: StateIndex<I>,
    owner: TwoPlayer,
    reaching_player: TwoPlayer,
) {
    if owner == reaching_player {
        buffer.reset_reaching_player(state);
    } else {
        buffer.reset_avoiding_player(state);
    }
}
