use super::algorithm_collections::{ChangeableOwners, NonstochasticGameAlgorithm};
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadInitialStates, ReadOwners, ReadPredecessors, ReadStateSpace,
};
use typed_index_collections::{Index, To1};

pub trait SolvableNonstochasticGame {
    type StateIdx: Index;

    type Model;

    fn set_owner(&mut self, state: Self::StateIdx, owner: TwoPlayer);

    fn get_winner(&mut self) -> TwoPlayer;

    fn get_winning_region(&mut self) -> To1<Self::StateIdx, bool>;

    fn get_game(&self) -> &Self::Model;
}

pub struct NonstochasticGameAndSolverExternalOwners<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
    AC: NonstochasticGameAlgorithm<
            <M as ReadStateSpace>::StateIdx,
            ModelContext: ChangeableOwners<<M as ReadStateSpace>::StateIdx>,
        >,
> {
    game: M,
    solver: AC,
    context: AC::ModelContext,
}

impl<
    M: ReadStateSpace
        + ReadOwners<StateIdx = <M as ReadStateSpace>::StateIdx, OwnerType = TwoPlayer>
        + ReadAtomicPropositions<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            APIdx = <AC as NonstochasticGameAlgorithm<<M as ReadStateSpace>::StateIdx>>::APIdx,
        > + ReadInitialStates<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
    AC: NonstochasticGameAlgorithm<
            <M as ReadStateSpace>::StateIdx,
            ModelContext: ChangeableOwners<<M as ReadStateSpace>::StateIdx>,
        >,
> NonstochasticGameAndSolverExternalOwners<M, AC>
{
    pub fn new(game: M, solver: AC) -> Self {
        let context = solver.create_model_context(&game);
        Self {
            game,
            solver,
            context,
        }
    }
}

impl<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
    AC: NonstochasticGameAlgorithm<
            <M as ReadStateSpace>::StateIdx,
            ModelContext: ChangeableOwners<<M as ReadStateSpace>::StateIdx>,
        >,
> SolvableNonstochasticGame for NonstochasticGameAndSolverExternalOwners<M, AC>
{
    type StateIdx = <M as ReadStateSpace>::StateIdx;
    type Model = M;

    fn set_owner(&mut self, state: Self::StateIdx, owner: TwoPlayer) {
        self.context.set_owner(state, owner);
    }

    fn get_winner(&mut self) -> TwoPlayer {
        self.solver
            .winning_with_context(&self.game, &mut self.context)
    }

    fn get_winning_region(&mut self) -> To1<Self::StateIdx, bool> {
        self.solver
            .winning_region_with_context(&self.game, &mut self.context)
    }

    fn get_game(&self) -> &Self::Model {
        &self.game
    }
}
