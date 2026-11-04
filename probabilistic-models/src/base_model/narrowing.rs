use crate::base_model::{
    BaseModel, BranchGraph, ChoiceBranchGraph, ChoiceGraph, Dtmc, Mdp, NonstochasticGame,
    StochasticGame, TransitionSystem,
};
use crate::{BranchIndex, ChoiceIndex, Ix, StateIndex};
use typed_index_collections::{Csr, Index, RawIndex, To1};

// Conversions move the underlying vectors and only reinterpret index types. All state, choice and
// branch indices are preserved, so other model components (rewards, labels, ...) stay valid.

// Checks that the i-th CSR entry ends at i + 1, i.e. every key has exactly one value.
fn all_ranges_have_one_entry<T: Index>(ends: &[T]) -> bool {
    ends.iter()
        .enumerate()
        .all(|(i, end)| end.raw().as_usize() == i + 1)
}

fn probabilities_one<I: Ix>(len: usize) -> To1<BranchIndex<I>, f64> {
    To1::with_entries(vec![1.0; len])
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

impl<I: Ix> BranchGraph<I> {
    pub fn every_state_has_one_branch(&self) -> bool {
        all_ranges_have_one_entry(self.state_to_branch.entries_raw())
    }

    pub fn try_into_choice_graph(self) -> Result<ChoiceGraph<I>, Self> {
        if !self.every_state_has_one_branch() {
            return Err(self);
        }
        Ok(ChoiceGraph {
            state_to_choice: Csr::identity(self.state_to_branch.entries_raw().len()),
            choice_destinations: self.branch_destinations.change_key_type::<ChoiceIndex<I>>(),
        })
    }

    pub fn into_choice_branch_graph(self) -> ChoiceBranchGraph<I> {
        ChoiceBranchGraph {
            state_to_choice: Csr::identity(self.state_to_branch.entries_raw().len()),
            choice_to_branch: self.state_to_branch.change_key_type::<ChoiceIndex<I>>(),
            branch_probabilities: self.branch_probabilities,
            branch_destinations: self.branch_destinations,
        }
    }
}

impl<I: Ix> ChoiceGraph<I> {
    pub fn every_state_has_one_choice(&self) -> bool {
        all_ranges_have_one_entry(self.state_to_choice.entries_raw())
    }

    pub fn try_into_branch_graph(self) -> Result<BranchGraph<I>, Self> {
        if !self.every_state_has_one_choice() {
            return Err(self);
        }
        let num_states = self.state_to_choice.entries_raw().len();
        Ok(BranchGraph {
            state_to_branch: Csr::identity(num_states),
            branch_probabilities: probabilities_one(num_states),
            branch_destinations: self.choice_destinations.change_key_type::<BranchIndex<I>>(),
        })
    }

    pub fn into_choice_branch_graph(self) -> ChoiceBranchGraph<I> {
        let num_choices = self.choice_destinations.len();
        ChoiceBranchGraph {
            state_to_choice: self.state_to_choice,
            choice_to_branch: Csr::identity(num_choices),
            branch_probabilities: probabilities_one(num_choices),
            branch_destinations: self.choice_destinations.change_key_type::<BranchIndex<I>>(),
        }
    }
}

impl<G, P> BaseModel<G, P> {
    fn map_graph<G2>(self, map: impl FnOnce(G) -> G2) -> BaseModel<G2, P> {
        BaseModel {
            graph: map(self.graph),
            players: self.players,
        }
    }

    fn try_map_graph<G2>(
        self,
        map: impl FnOnce(G) -> Result<G2, G>,
    ) -> Result<BaseModel<G2, P>, Self> {
        match map(self.graph) {
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

impl<I: Ix, P> BaseModel<BranchGraph<I>, P> {
    pub fn try_into_choice_graph(self) -> Result<BaseModel<ChoiceGraph<I>, P>, Self> {
        self.try_map_graph(BranchGraph::try_into_choice_graph)
    }

    pub fn into_choice_branch_graph(self) -> BaseModel<ChoiceBranchGraph<I>, P> {
        self.map_graph(BranchGraph::into_choice_branch_graph)
    }
}

impl<I: Ix, P> BaseModel<ChoiceGraph<I>, P> {
    pub fn try_into_branch_graph(self) -> Result<BaseModel<BranchGraph<I>, P>, Self> {
        self.try_map_graph(ChoiceGraph::try_into_branch_graph)
    }

    pub fn into_choice_branch_graph(self) -> BaseModel<ChoiceBranchGraph<I>, P> {
        self.map_graph(ChoiceGraph::into_choice_branch_graph)
    }
}

impl<I: Ix, P> BaseModel<ChoiceBranchGraph<I>, P> {
    pub fn try_into_choice_graph(self) -> Result<BaseModel<ChoiceGraph<I>, P>, Self> {
        self.try_map_graph(ChoiceBranchGraph::try_into_choice_graph)
    }

    pub fn try_into_branch_graph(self) -> Result<BaseModel<BranchGraph<I>, P>, Self> {
        self.try_map_graph(ChoiceBranchGraph::try_into_branch_graph)
    }
}

impl<I: Ix> TransitionSystem<I> {
    // Possible iff every state has exactly one choice. Indices are preserved.
    pub fn try_into_dtmc(self) -> Result<Dtmc<I>, Self> {
        self.try_into_branch_graph()
    }

    pub fn into_mdp(self) -> Mdp<I> {
        self.into_choice_branch_graph()
    }
}

impl<I: Ix> Dtmc<I> {
    pub fn try_into_transition_system(self) -> Result<TransitionSystem<I>, Self> {
        self.try_into_choice_graph()
    }

    pub fn into_mdp(self) -> Mdp<I> {
        self.into_choice_branch_graph()
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
}

impl<I: Ix> NonstochasticGame<I> {
    pub fn into_stochastic_game(self) -> StochasticGame<I> {
        self.into_choice_branch_graph()
    }
}

impl<I: Ix> StochasticGame<I> {
    // Possible iff every choice has exactly one branch. Indices and owners are preserved.
    pub fn try_into_nonstochastic_game(self) -> Result<NonstochasticGame<I>, Self> {
        self.try_into_choice_graph()
    }
}
