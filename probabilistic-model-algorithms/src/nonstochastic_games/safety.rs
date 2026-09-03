use crate::attractor;
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadOwners, ReadPredecessors, ReadStateSpace,
};
use typed_index_collections::{Index, To1};

const REACHING_PLAYER: TwoPlayer = TwoPlayer::Adam;

pub struct SafetyContext<StateIdx: Index> {
    bad_states: Vec<StateIdx>,
    owners: To1<StateIdx, TwoPlayer>,
    buffer: attractor::AttractorBuffer<StateIdx>,
    dirty: bool,
}

impl<StateIdx: Index> SafetyContext<StateIdx> {
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
            "safety context is dirty, call reset() or clear_dirty_flag() before reusing it"
        );
        self.dirty = true;
    }
}

pub fn create_safety_context<
    M: ReadStateSpace
        + ReadOwners<StateIdx = <M as ReadStateSpace>::StateIdx, OwnerType = TwoPlayer>
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    good_states: <M as ReadAtomicPropositions>::APIdx,
) -> SafetyContext<<M as ReadStateSpace>::StateIdx> {
    let mut context = SafetyContext {
        bad_states: super::states_without_ap(model, good_states),
        owners: super::model_owners(model),
        buffer: attractor::AttractorBuffer::create(model),
        dirty: false,
    };
    context.reset();
    context
}

pub fn solve_safety<
    M: ReadStateSpace
        + ReadOwners<StateIdx = <M as ReadStateSpace>::StateIdx, OwnerType = TwoPlayer>
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    good_states: <M as ReadAtomicPropositions>::APIdx,
) -> To1<<M as ReadStateSpace>::StateIdx, bool> {
    let mut context = create_safety_context(model, good_states);
    solve_safety_raw(model, &mut context)
}

pub fn solve_safety_raw<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    context: &mut SafetyContext<<M as ReadStateSpace>::StateIdx>,
) -> To1<<M as ReadStateSpace>::StateIdx, bool> {
    context.mark_dirty();
    attractor::attractor_with_buffer(
        model,
        context.bad_states.iter().cloned(),
        &mut context.buffer,
    )
    .map(|&attracted| !attracted)
}

pub fn safety_winner_from_state<
    M: ReadStateSpace
        + ReadOwners<StateIdx = <M as ReadStateSpace>::StateIdx, OwnerType = TwoPlayer>
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    good_states: <M as ReadAtomicPropositions>::APIdx,
    state: <M as ReadStateSpace>::StateIdx,
) -> TwoPlayer {
    let mut context = create_safety_context(model, good_states);
    safety_winner_from_state_raw(model, &mut context, state)
}

pub fn safety_winner_from_state_raw<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    context: &mut SafetyContext<<M as ReadStateSpace>::StateIdx>,
    state: <M as ReadStateSpace>::StateIdx,
) -> TwoPlayer {
    context.mark_dirty();
    match attractor::attractor_contains_state_with_buffer(
        model,
        context.bad_states.iter().cloned(),
        state,
        &mut context.buffer,
    ) {
        true => TwoPlayer::Adam,
        false => TwoPlayer::Eve,
    }
}
