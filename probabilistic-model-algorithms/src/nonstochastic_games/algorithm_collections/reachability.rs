use super::NonstochasticGameAlgorithm;
use crate::attractor;
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadInitialStates, ReadOwners, ReadPredecessors, ReadStateSpace,
    StateSet,
};
use typed_index_collections::{Index, To1};

pub struct ReachabilityAlgorithmCollection<APIdx: Index> {
    target_states: APIdx,
}

impl<APIdx: Index> ReachabilityAlgorithmCollection<APIdx> {
    pub fn new(target_states: APIdx) -> Self {
        Self { target_states }
    }
}

impl<StateIdx: Index, APIdx: Index> NonstochasticGameAlgorithm<StateIdx>
    for ReachabilityAlgorithmCollection<APIdx>
{
    type APIdx = APIdx;
    type ModelContext = ReachabilityAlgorithmContext<StateIdx>;

    fn create_model_context<
        M: ReadStateSpace<StateIdx = StateIdx>
            + ReadOwners<StateIdx = StateIdx, OwnerType = TwoPlayer>
            + ReadAtomicPropositions<StateIdx = StateIdx, APIdx = Self::APIdx>
            + ReadInitialStates<StateIdx = StateIdx>,
    >(
        &self,
        model: &M,
    ) -> Self::ModelContext {
        let mut initial_states = model.initial_states().iter();
        let initial_state = initial_states
            .next()
            .expect("Expected the model to have exactly one initial state");
        assert!(initial_states.next().is_none());

        let target_states: Vec<StateIdx> = model
            .states()
            .into_iter()
            .filter(|&state| model.is_atomic_proposition_set(state, self.target_states))
            .collect();
        let mut buffer = attractor::AttractorBuffer::create(model);
        buffer.reset_owner_counts(model, TwoPlayer::Eve);
        ReachabilityAlgorithmContext {
            target_states,
            buffer,
            initial_state,
        }
    }

    fn winning_with_context<
        M: ReadStateSpace<StateIdx = StateIdx>
            + ReadPredecessors<
                StateIdx = StateIdx,
                ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
                BranchIdx = <M as ReadStateSpace>::BranchIdx,
            >,
    >(
        &mut self,
        model: &M,
        context: &mut Self::ModelContext,
    ) -> TwoPlayer {
        self.winning_from_state_with_context(model, context.initial_state, context)
    }

    fn winning_from_state_with_context<
        M: ReadStateSpace<StateIdx = StateIdx>
            + ReadPredecessors<
                StateIdx = StateIdx,
                ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
                BranchIdx = <M as ReadStateSpace>::BranchIdx,
            >,
    >(
        &mut self,
        model: &M,
        state: StateIdx,
        context: &mut Self::ModelContext,
    ) -> TwoPlayer {
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

    fn winning_region_with_context<
        M: ReadStateSpace<StateIdx = StateIdx>
            + ReadPredecessors<
                StateIdx = StateIdx,
                ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
                BranchIdx = <M as ReadStateSpace>::BranchIdx,
            >,
    >(
        &mut self,
        model: &M,
        context: &mut Self::ModelContext,
    ) -> To1<StateIdx, bool> {
        attractor::attractor_with_buffer(
            model,
            context.target_states.iter().cloned(),
            &mut context.buffer,
        )
    }
}

pub struct ReachabilityAlgorithmContext<StateIdx: Index> {
    target_states: Vec<StateIdx>,
    buffer: attractor::AttractorBuffer<StateIdx>,
    initial_state: StateIdx,
}

impl<StateIdx: Index> super::ChangeableOwners<StateIdx> for ReachabilityAlgorithmContext<StateIdx> {
    fn set_owner(&mut self, index: StateIdx, owner: TwoPlayer) {
        match owner {
            TwoPlayer::Eve => self.buffer.reset_reaching_player(index),
            TwoPlayer::Adam => self.buffer.reset_avoiding_player(index),
        }
    }
}
