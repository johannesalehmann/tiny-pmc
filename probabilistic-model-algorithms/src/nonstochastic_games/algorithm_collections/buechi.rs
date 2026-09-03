use super::NonstochasticGameAlgorithm;
use crate::attractor;
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadInitialStates, ReadOwners, ReadPredecessors, ReadStateSpace,
    StateSet,
};
use typed_index_collections::{Index, To1, ValuePerIndexSource};

pub struct BuechiAlgorithmCollection<APIdx: Index> {
    buechi_states: APIdx,
}

impl<APIdx: Index> BuechiAlgorithmCollection<APIdx> {
    pub fn new(buechi_states: APIdx) -> Self {
        Self { buechi_states }
    }
}

impl<StateIdx: Index, APIdx: Index> NonstochasticGameAlgorithm<StateIdx>
    for BuechiAlgorithmCollection<APIdx>
{
    type APIdx = APIdx;
    type ModelContext = BuechiAlgorithmContext<StateIdx>;

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

        let buechi_states: Vec<StateIdx> = model
            .states()
            .into_iter()
            .filter(|&state| model.is_atomic_proposition_set(state, self.buechi_states))
            .collect();
        let mut buffer = attractor::AttractorBuffer::create(model);
        buffer.reset_owner_counts(model, TwoPlayer::Eve);

        let mut owners = To1::with_capacity(model.states().len());
        for state in model.states() {
            owners.add_checked(state, model.state_owner(state));
        }

        BuechiAlgorithmContext {
            buechi_states,
            buffer,
            owners,
            unreachable: To1::with_entries(vec![false; model.states().len()]),
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
        let winning_region = self.winning_region_with_context(model, context);
        if winning_region[state] {
            TwoPlayer::Eve
        } else {
            TwoPlayer::Adam
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
        let mut changed = true;

        context.unreachable.fill(false);

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

            let player_2_reachable = attractor::attractor_with_buffer(
                model,
                reachable.false_values().into_iter(),
                &mut context.buffer,
            );

            changed = context.add_new_unreachable_states(&player_2_reachable);
        }

        context.unreachable.map(|&unreachable| !unreachable)
    }
}

pub struct BuechiAlgorithmContext<StateIdx: Index> {
    buechi_states: Vec<StateIdx>,
    buffer: attractor::AttractorBuffer<StateIdx>,
    owners: To1<StateIdx, TwoPlayer>,
    unreachable: To1<StateIdx, bool>,
    initial_state: StateIdx,
}

impl<StateIdx: Index> BuechiAlgorithmContext<StateIdx> {
    fn reset_buffer(&mut self, reaching_player: TwoPlayer) {
        for (index, &owner) in self.owners.enumerate() {
            if self.unreachable[index] {
                self.buffer.reset_sink_state(index);
            } else if reaching_player == owner {
                self.buffer.reset_reaching_player(index);
            } else {
                self.buffer.reset_avoiding_player(index);
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

impl<StateIdx: Index> super::ChangeableOwners<StateIdx> for BuechiAlgorithmContext<StateIdx> {
    fn set_owner(&mut self, index: StateIdx, owner: TwoPlayer) {
        self.owners[index] = owner;
    }
}
