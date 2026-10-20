use crate::attractor;
use probabilistic_models::base_model::{ReadOwners, TwoPlayer};
use probabilistic_models::traits::{ReadAtomicPropositions, ReadPredecessors, ReadStateSpace};
use probabilistic_models::{AtomicPropositionIndex, Ix, StateIndex};
use typed_index_collections::To1;

const REACHING_PLAYER: TwoPlayer = TwoPlayer::Eve;

pub struct ReachabilityContext<I: Ix> {
    target_states: Vec<StateIndex<I>>,
    owners: To1<StateIndex<I>, TwoPlayer>,
    buffer: attractor::AttractorBuffer<I>,
    dirty: bool,
}

impl<I: Ix> ReachabilityContext<I> {
    pub fn owner(&self, state: StateIndex<I>) -> TwoPlayer {
        self.owners[state]
    }

    pub fn set_owner(&mut self, state: StateIndex<I>, owner: TwoPlayer) {
        self.owners[state] = owner;
        super::set_owner_count(&mut self.buffer, state, owner, REACHING_PLAYER);
    }

    pub fn reset(&mut self) {
        super::reset_owner_counts(&self.owners, &mut self.buffer, REACHING_PLAYER);
        self.dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty_flag(&mut self) {
        self.dirty = false;
    }

    fn mark_dirty(&mut self) {
        assert!(
            !self.dirty,
            "reachability context is dirty, call reset() or clear_dirty_flag() before reusing it"
        );
        self.dirty = true;
    }
}

pub fn create_reachability_context<
    M: ReadStateSpace + ReadOwners<OwnerType = TwoPlayer> + ReadAtomicPropositions,
>(
    model: &M,
    target_states: AtomicPropositionIndex<M::Ix>,
) -> ReachabilityContext<M::Ix> {
    let mut context = ReachabilityContext {
        target_states: super::states_with_ap(model, target_states),
        owners: super::model_owners(model),
        buffer: attractor::AttractorBuffer::create(model),
        dirty: false,
    };
    context.reset();
    context
}

pub fn solve_reachability<
    M: ReadStateSpace + ReadOwners<OwnerType = TwoPlayer> + ReadAtomicPropositions + ReadPredecessors,
>(
    model: &M,
    target_states: AtomicPropositionIndex<M::Ix>,
) -> To1<StateIndex<M::Ix>, bool> {
    let mut context = create_reachability_context(model, target_states);
    solve_reachability_raw(model, &mut context)
}

pub fn solve_reachability_raw<M: ReadStateSpace + ReadPredecessors>(
    model: &M,
    context: &mut ReachabilityContext<M::Ix>,
) -> To1<StateIndex<M::Ix>, bool> {
    context.mark_dirty();
    attractor::attractor_with_buffer(
        model,
        context.target_states.iter().cloned(),
        &mut context.buffer,
    )
}

pub fn reachability_winner_from_state<
    M: ReadStateSpace + ReadOwners<OwnerType = TwoPlayer> + ReadAtomicPropositions + ReadPredecessors,
>(
    model: &M,
    target_states: AtomicPropositionIndex<M::Ix>,
    state: StateIndex<M::Ix>,
) -> TwoPlayer {
    let mut context = create_reachability_context(model, target_states);
    reachability_winner_from_state_raw(model, &mut context, state)
}

pub fn reachability_winner_from_state_raw<M: ReadStateSpace + ReadPredecessors>(
    model: &M,
    context: &mut ReachabilityContext<M::Ix>,
    state: StateIndex<M::Ix>,
) -> TwoPlayer {
    context.mark_dirty();
    match attractor::attractor_contains_state_with_buffer(
        model,
        context.target_states.iter().cloned(),
        state,
        &mut context.buffer,
    ) {
        true => TwoPlayer::Eve,
        false => TwoPlayer::Adam,
    }
}
