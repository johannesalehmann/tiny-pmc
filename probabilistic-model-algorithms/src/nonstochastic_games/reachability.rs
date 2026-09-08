use crate::attractor;
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadOwners, ReadPredecessors, ReadStateSpace,
};
use typed_index_collections::{Index, To1};

const REACHING_PLAYER: TwoPlayer = TwoPlayer::Eve;

pub struct ReachabilityContext<StateIdx: Index> {
    target_states: Vec<StateIdx>,
    owners: To1<StateIdx, TwoPlayer>,
    buffer: attractor::AttractorBuffer<StateIdx>,
    dirty: bool,
}

impl<StateIdx: Index> ReachabilityContext<StateIdx> {
    pub fn owner(&self, state: StateIdx) -> TwoPlayer {
        self.owners[state]
    }

    pub fn set_owner(&mut self, state: StateIdx, owner: TwoPlayer) {
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
    M: ReadStateSpace
        + ReadOwners<StateIdx = M::StateIndex, OwnerType = TwoPlayer>
        + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    target_states: <M as ReadAtomicPropositions>::APIdx,
) -> ReachabilityContext<M::StateIndex> {
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
    M: ReadStateSpace
        + ReadOwners<StateIdx = M::StateIndex, OwnerType = TwoPlayer>
        + ReadAtomicPropositions<StateIdx = M::StateIndex>
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        >,
>(
    model: &M,
    target_states: <M as ReadAtomicPropositions>::APIdx,
) -> To1<M::StateIndex, bool> {
    let mut context = create_reachability_context(model, target_states);
    solve_reachability_raw(model, &mut context)
}

pub fn solve_reachability_raw<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        >,
>(
    model: &M,
    context: &mut ReachabilityContext<M::StateIndex>,
) -> To1<M::StateIndex, bool> {
    context.mark_dirty();
    attractor::attractor_with_buffer(
        model,
        context.target_states.iter().cloned(),
        &mut context.buffer,
    )
}

pub fn reachability_winner_from_state<
    M: ReadStateSpace
        + ReadOwners<StateIdx = M::StateIndex, OwnerType = TwoPlayer>
        + ReadAtomicPropositions<StateIdx = M::StateIndex>
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        >,
>(
    model: &M,
    target_states: <M as ReadAtomicPropositions>::APIdx,
    state: M::StateIndex,
) -> TwoPlayer {
    let mut context = create_reachability_context(model, target_states);
    reachability_winner_from_state_raw(model, &mut context, state)
}

pub fn reachability_winner_from_state_raw<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        >,
>(
    model: &M,
    context: &mut ReachabilityContext<M::StateIndex>,
    state: M::StateIndex,
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
