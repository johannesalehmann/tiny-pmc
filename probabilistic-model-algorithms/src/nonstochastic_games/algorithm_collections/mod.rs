mod reachability;
pub use reachability::{ReachabilityAlgorithmCollection, ReachabilityAlgorithmContext};

mod safety;
pub use safety::{SafetyAlgorithmCollection, SafetyAlgorithmContext};

mod buechi;
pub use buechi::{BuechiAlgorithmCollection, BuechiAlgorithmContext};

use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadInitialStates, ReadOwners, ReadPredecessors, ReadStateSpace,
};
use typed_index_collections::{Index, To1};

pub trait NonstochasticGameAlgorithm<StateIdx: Index>: Sized {
    type APIdx: Index;
    type ModelContext: Sized;

    fn create_model_context<
        M: ReadStateSpace<StateIdx = StateIdx>
            + ReadOwners<StateIdx = StateIdx, OwnerType = TwoPlayer>
            + ReadAtomicPropositions<StateIdx = StateIdx, APIdx = Self::APIdx>
            + ReadInitialStates<StateIdx = StateIdx>,
    >(
        &self,
        model: &M,
    ) -> Self::ModelContext;

    fn winning<
        M: ReadStateSpace<StateIdx = StateIdx>
            + ReadOwners<StateIdx = StateIdx, OwnerType = TwoPlayer>
            + ReadAtomicPropositions<StateIdx = StateIdx, APIdx = Self::APIdx>
            + ReadInitialStates<StateIdx = StateIdx>
            + ReadPredecessors<
                StateIdx = StateIdx,
                ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
                BranchIdx = <M as ReadStateSpace>::BranchIdx,
            >,
    >(
        &mut self,
        model: &M,
    ) -> TwoPlayer {
        let mut context = self.create_model_context(model);
        self.winning_with_context(model, &mut context)
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
    ) -> TwoPlayer;

    fn winning_from_state<
        M: ReadStateSpace<StateIdx = StateIdx>
            + ReadOwners<StateIdx = StateIdx, OwnerType = TwoPlayer>
            + ReadAtomicPropositions<StateIdx = StateIdx, APIdx = Self::APIdx>
            + ReadInitialStates<StateIdx = StateIdx>
            + ReadPredecessors<
                StateIdx = StateIdx,
                ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
                BranchIdx = <M as ReadStateSpace>::BranchIdx,
            >,
    >(
        &mut self,
        model: &M,
        state: StateIdx,
    ) -> TwoPlayer {
        let mut context = self.create_model_context(model);
        self.winning_from_state_with_context(model, state, &mut context)
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
    ) -> TwoPlayer;

    fn winning_region<
        M: ReadStateSpace<StateIdx = StateIdx>
            + ReadOwners<StateIdx = StateIdx, OwnerType = TwoPlayer>
            + ReadAtomicPropositions<StateIdx = StateIdx, APIdx = Self::APIdx>
            + ReadInitialStates<StateIdx = StateIdx>
            + ReadPredecessors<
                StateIdx = StateIdx,
                ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
                BranchIdx = <M as ReadStateSpace>::BranchIdx,
            >,
    >(
        &mut self,
        model: &M,
    ) -> To1<StateIdx, bool> {
        let mut context = self.create_model_context(model);
        self.winning_region_with_context(model, &mut context)
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
    ) -> To1<StateIdx, bool>;
}

pub trait ChangeableOwners<StateIdx: Index> {
    fn set_owner(&mut self, index: StateIdx, owner: TwoPlayer);
}
