use crate::attractor;
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadOwners, ReadPredecessors, ReadStateSpace,
};
use typed_index_collections::{Index, To1, ValuePerIndexSource};

pub struct BuechiContext<StateIdx: Index> {
    buechi_states: Vec<StateIdx>,
    owners: To1<StateIdx, TwoPlayer>,
    buffer: attractor::AttractorBuffer<StateIdx>,
    unreachable: To1<StateIdx, bool>,
    dirty: bool,
}

impl<StateIdx: Index> BuechiContext<StateIdx> {
    pub fn owner(&self, state: StateIdx) -> TwoPlayer {
        self.owners[state]
    }

    pub fn set_owner(&mut self, state: StateIdx, owner: TwoPlayer) {
        self.owners[state] = owner;
    }

    pub fn reset(&mut self) {
        self.unreachable.fill(false);
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
            "buechi context is dirty, call reset() or clear_dirty_flag() before reusing it"
        );
        self.dirty = true;
    }

    fn reset_buffer(&mut self, reaching_player: TwoPlayer) {
        for (state, &owner) in self.owners.enumerate() {
            if self.unreachable[state] {
                self.buffer.reset_sink_state(state);
            } else {
                super::set_owner_count(&mut self.buffer, state, owner, reaching_player);
            }
        }
    }

    fn add_new_unreachable_states(&mut self, new_unreachable: &To1<StateIdx, bool>) -> bool {
        let mut changed = false;
        for state in self.unreachable.keys() {
            if !self.unreachable[state] && new_unreachable[state] {
                self.unreachable[state] = true;
                changed = true;
            }
        }
        changed
    }
}

pub fn create_buechi_context<
    M: ReadStateSpace
        + ReadOwners<StateIdx = <M as ReadStateSpace>::StateIdx, OwnerType = TwoPlayer>
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    buechi_states: <M as ReadAtomicPropositions>::APIdx,
) -> BuechiContext<<M as ReadStateSpace>::StateIdx> {
    BuechiContext {
        buechi_states: super::states_with_ap(model, buechi_states),
        owners: super::model_owners(model),
        buffer: attractor::AttractorBuffer::create(model),
        unreachable: To1::with_entries(vec![false; model.states().len()]),
        dirty: false,
    }
}

pub fn solve_buechi<
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
    buechi_states: <M as ReadAtomicPropositions>::APIdx,
) -> To1<<M as ReadStateSpace>::StateIdx, bool> {
    let mut context = create_buechi_context(model, buechi_states);
    solve_buechi_raw(model, &mut context)
}

pub fn solve_buechi_raw<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    context: &mut BuechiContext<<M as ReadStateSpace>::StateIdx>,
) -> To1<<M as ReadStateSpace>::StateIdx, bool> {
    context.mark_dirty();

    let mut changed = true;

    while changed {
        context.reset_buffer(TwoPlayer::Eve);

        let reachable = attractor::attractor_with_buffer(
            model,
            context
                .buechi_states
                .iter()
                .filter(|state| !context.unreachable[**state])
                .cloned(),
            &mut context.buffer,
        );

        context.reset_buffer(TwoPlayer::Adam);

        let adam_reachable = attractor::attractor_with_buffer(
            model,
            reachable.false_values().into_iter(),
            &mut context.buffer,
        );

        changed = context.add_new_unreachable_states(&adam_reachable);
    }

    context.unreachable.map(|&unreachable| !unreachable)
}

pub fn buechi_winner_from_state<
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
    buechi_states: <M as ReadAtomicPropositions>::APIdx,
    state: <M as ReadStateSpace>::StateIdx,
) -> TwoPlayer {
    let mut context = create_buechi_context(model, buechi_states);
    buechi_winner_from_state_raw(model, &mut context, state)
}

pub fn buechi_winner_from_state_raw<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    context: &mut BuechiContext<<M as ReadStateSpace>::StateIdx>,
    state: <M as ReadStateSpace>::StateIdx,
) -> TwoPlayer {
    if solve_buechi_raw(model, context)[state] {
        TwoPlayer::Eve
    } else {
        TwoPlayer::Adam
    }
}
