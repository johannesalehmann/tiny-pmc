use crate::traits::{
    BuildBranchGraph, BuildChoiceBranchGraph, BuildChoiceGraph, BuildStates, ClearModel,
    ReadChoiceDestinations, ReadStateBranches, ReadStateSpace, derive_build_branch_graph,
    derive_build_choice_branch_graph, derive_build_choice_graph, derive_build_states,
    derive_read_choice_destinations, derive_read_state_branches, derive_read_state_space,
};
use typed_index_collections::Index;

mod graph;
pub use graph::{BranchGraph, ChoiceBranchGraph, ChoiceGraph};

mod owners;
#[allow(unused)]
pub(crate) use owners::derive_read_owners;
pub use owners::{ReadOwners, SinglePlayer, TurnBased, TwoPlayer};

mod mdp_macro;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct BaseModel<Graph, Players> {
    pub graph: Graph,
    pub players: Players,
}

pub type TransitionSystem<StateIdx, ChoiceIdx> =
    BaseModel<ChoiceGraph<StateIdx, ChoiceIdx>, SinglePlayer>;
pub type Dtmc<StateIdx, BranchIdx> = BaseModel<BranchGraph<StateIdx, BranchIdx>, SinglePlayer>;
pub type Mdp<StateIdx, ChoiceIdx, BranchIdx> =
    BaseModel<ChoiceBranchGraph<StateIdx, ChoiceIdx, BranchIdx>, SinglePlayer>;

pub type NonstochasticGame<StateIdx, ChoiceIdx> =
    BaseModel<ChoiceGraph<StateIdx, ChoiceIdx>, TurnBased<StateIdx, TwoPlayer>>;
pub type StochasticGame<StateIdx, ChoiceIdx, BranchIdx> =
    BaseModel<ChoiceBranchGraph<StateIdx, ChoiceIdx, BranchIdx>, TurnBased<StateIdx, TwoPlayer>>;

impl<StateIdx: Index, ChoiceIdx: Index> TransitionSystem<StateIdx, ChoiceIdx> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<StateIdx: Index, BranchIdx: Index> Dtmc<StateIdx, BranchIdx> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> Mdp<StateIdx, ChoiceIdx, BranchIdx> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<G> BaseModel<G, SinglePlayer> {
    pub fn with_graph(graph: G) -> Self {
        Self {
            graph,
            players: SinglePlayer,
        }
    }
}

impl<StateIdx: Index, ChoiceIdx: Index> NonstochasticGame<StateIdx, ChoiceIdx> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index>
    StochasticGame<StateIdx, ChoiceIdx, BranchIdx>
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<G: ReadStateSpace, P> ReadStateSpace for BaseModel<G, P> {
    type StateIndex = G::StateIndex;
    type ChoiceIndex = G::ChoiceIndex;
    type BranchIndex = G::BranchIndex;

    derive_read_state_space!(graph);
}

impl<G: ReadStateBranches, P> ReadStateBranches for BaseModel<G, P> {
    derive_read_state_branches!(graph);
}

impl<G: ReadChoiceDestinations, P> ReadChoiceDestinations for BaseModel<G, P> {
    derive_read_choice_destinations!(graph);
}

impl<G: ReadStateSpace> ReadOwners for BaseModel<G, SinglePlayer> {
    type OwnerType = SinglePlayer;
    type StateIdx = G::StateIndex;

    fn state_owner(&self, _state: Self::StateIdx) -> Self::OwnerType {
        SinglePlayer
    }
}

impl<G: ReadStateSpace, Owner: Copy> ReadOwners for BaseModel<G, TurnBased<G::StateIndex, Owner>> {
    type OwnerType = Owner;
    type StateIdx = G::StateIndex;

    fn state_owner(&self, state: Self::StateIdx) -> Self::OwnerType {
        self.players.state_to_owner[state]
    }
}

// TODO: Provide analogous functions for non-singleplayer base models
impl<G: BuildStates> BuildStates for BaseModel<G, SinglePlayer> {
    derive_build_states!(graph);
}

impl<G: BuildStates, Owner> BaseModel<G, TurnBased<G::StateIndex, Owner>> {
    pub fn add_state(&mut self, owner: Owner) -> G::StateIndex {
        let state = self.graph.add_state();
        self.players.state_to_owner.add_checked(state, owner);
        state
    }
}

impl<G: BuildChoiceBranchGraph, P> BuildChoiceBranchGraph for BaseModel<G, P> {
    derive_build_choice_branch_graph!(graph);
}

impl<G: BuildChoiceGraph, P> BuildChoiceGraph for BaseModel<G, P> {
    derive_build_choice_graph!(graph);
}

impl<G: BuildBranchGraph, P> BuildBranchGraph for BaseModel<G, P> {
    derive_build_branch_graph!(graph);
}

impl<G: ClearModel, P: ClearModel> ClearModel for BaseModel<G, P> {
    fn clear(&mut self) {
        self.graph.clear();
        self.players.clear();
    }
}
