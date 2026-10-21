use crate::base_model::{
    AnyBaseModel, BaseModel, BaseModelType, BranchGraph, ChoiceBranchGraph, ChoiceGraph, Dtmc, Mdp,
    NonstochasticGame, StochasticGame, TransitionSystem,
};
use crate::{ChoiceIndex, Ix, StateIndex};
use typed_index_collections::{Index, RawIndex};

// Narrowing moves the underlying vectors and only reinterprets index types. All state, choice and
// branch indices are preserved, so other model components (rewards, labels, ...) stay valid.

// Checks that the i-th CSR entry ends at i + 1, i.e. every key has exactly one value.
fn all_ranges_have_one_entry<T: Index>(ends: &[T]) -> bool {
    ends.iter()
        .enumerate()
        .all(|(i, end)| end.raw().as_usize() == i + 1)
}

impl<I: Ix> ChoiceBranchGraph<I> {
    pub fn every_state_has_one_choice(&self) -> bool {
        all_ranges_have_one_entry(self.state_to_choice.entries_raw())
    }

    pub fn every_choice_has_one_branch(&self) -> bool {
        all_ranges_have_one_entry(self.choice_to_branch.entries_raw())
    }

    // Possible iff every choice has exactly one branch. Then the branch index equals the choice
    // index. Branches with duplicate destinations are not merged.
    pub fn try_into_choice_graph(self) -> Result<ChoiceGraph<I>, Self> {
        if !self.every_choice_has_one_branch() {
            return Err(self);
        }
        Ok(ChoiceGraph {
            state_to_choice: self.state_to_choice,
            choice_destinations: self.branch_destinations.change_key_type::<ChoiceIndex<I>>(),
        })
    }

    // Possible iff every state has exactly one choice. Then the choice index equals the state
    // index.
    pub fn try_into_branch_graph(self) -> Result<BranchGraph<I>, Self> {
        if !self.every_state_has_one_choice() {
            return Err(self);
        }
        Ok(BranchGraph {
            state_to_branch: self.choice_to_branch.change_key_type::<StateIndex<I>>(),
            branch_probabilities: self.branch_probabilities,
            branch_destinations: self.branch_destinations,
        })
    }
}

impl<I: Ix, P> BaseModel<ChoiceBranchGraph<I>, P> {
    pub fn try_into_choice_graph(self) -> Result<BaseModel<ChoiceGraph<I>, P>, Self> {
        match self.graph.try_into_choice_graph() {
            Ok(graph) => Ok(BaseModel {
                graph,
                players: self.players,
            }),
            Err(graph) => Err(BaseModel {
                graph,
                players: self.players,
            }),
        }
    }
    pub fn try_into_branch_graph(self) -> Result<BaseModel<BranchGraph<I>, P>, Self> {
        match self.graph.try_into_branch_graph() {
            Ok(graph) => Ok(BaseModel {
                graph,
                players: self.players,
            }),
            Err(graph) => Err(BaseModel {
                graph,
                players: self.players,
            }),
        }
    }
}

impl<I: Ix> Mdp<I> {
    // Possible iff every state has exactly one choice. Indices are preserved.
    pub fn try_into_dtmc(self) -> Result<Dtmc<I>, Self> {
        self.try_into_branch_graph()
    }

    // Possible iff every choice has exactly one branch. Indices are preserved.
    pub fn try_into_transition_system(self) -> Result<TransitionSystem<I>, Self> {
        self.try_into_choice_graph()
    }

    // Narrows to exactly the target type if the model fits it.
    pub fn try_narrow_to(self, target: BaseModelType) -> Result<AnyBaseModel<I>, Self> {
        match target {
            BaseModelType::TransitionSystem => self.try_into_transition_system().map(Into::into),
            BaseModelType::Dtmc => self.try_into_dtmc().map(Into::into),
            BaseModelType::Mdp => Ok(self.into()),
            BaseModelType::NonstochasticGame | BaseModelType::StochasticGame => Err(self),
        }
    }
}

impl<I: Ix> StochasticGame<I> {
    // Possible iff every choice has exactly one branch. Indices and owners are preserved.
    pub fn try_into_nonstochastic_game(self) -> Result<NonstochasticGame<I>, Self> {
        self.try_into_choice_graph()
    }

    // Narrows to exactly the target type if the model fits it. Only game targets are possible.
    pub fn try_narrow_to(self, target: BaseModelType) -> Result<AnyBaseModel<I>, Self> {
        match target {
            BaseModelType::NonstochasticGame => self.try_into_nonstochastic_game().map(Into::into),
            BaseModelType::StochasticGame => Ok(self.into()),
            BaseModelType::TransitionSystem | BaseModelType::Dtmc | BaseModelType::Mdp => Err(self),
        }
    }
}
