use crate::traits::{
    BuildBranchGraph, BuildChoiceBranchGraph, BuildChoiceGraph, BuildStates, ClearModel,
    ReadChoiceDestinations, ReadStateBranches, ReadStateSpace, derive_build_branch_graph,
    derive_build_choice_branch_graph, derive_build_choice_graph, derive_build_states,
    derive_read_choice_destinations, derive_read_state_branches, derive_read_state_space,
};
use crate::{DefaultIx, HasIx, Model, StateIndex};

mod graph;
pub use graph::{BranchGraph, ChoiceBranchGraph, ChoiceGraph};

mod owners;
pub use owners::{BuildOwners, ReadOwners, ReadPlayers, SinglePlayer, TurnBased, TwoPlayer};
pub(crate) use owners::{derive_read_owners, derive_read_players};

mod base_model_enum;
pub use base_model_enum::{AnyBaseModel, BaseModelEnum, BaseModelType};

mod narrowest;
mod narrowing;

mod model_macros;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct BaseModel<Graph, Players> {
    pub graph: Graph,
    pub players: Players,
}

impl<G: HasIx, P> HasIx for BaseModel<G, P> {
    type Ix = G::Ix;
}

pub type TransitionSystem<I = DefaultIx> = BaseModel<ChoiceGraph<I>, SinglePlayer>;
pub type Dtmc<I = DefaultIx> = BaseModel<BranchGraph<I>, SinglePlayer>;
pub type Mdp<I = DefaultIx> = BaseModel<ChoiceBranchGraph<I>, SinglePlayer>;

pub type NonstochasticGame<I = DefaultIx> = BaseModel<ChoiceGraph<I>, TurnBased<I>>;
pub type StochasticGame<I = DefaultIx> = BaseModel<ChoiceBranchGraph<I>, TurnBased<I>>;

// `new` creates models with the default index profile, so that `Mdp::new()` needs no type
// annotations (default type parameters are not used for inference). Models with other profiles
// are created via `default`, e.g. `Mdp::<Uniform<u64>>::default()`.
macro_rules! constructors {
    ($model:ident) => {
        impl $model {
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}

constructors!(TransitionSystem);
constructors!(Dtmc);
constructors!(Mdp);
constructors!(NonstochasticGame);
constructors!(StochasticGame);

impl<G> BaseModel<G, SinglePlayer> {
    pub fn with_graph(graph: G) -> Self {
        Self {
            graph,
            players: SinglePlayer,
        }
    }
}

impl<G: ReadStateSpace, P> ReadStateSpace for BaseModel<G, P> {
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

    fn state_owner(&self, _state: StateIndex<Self::Ix>) -> Self::OwnerType {
        SinglePlayer
    }
}

impl<G: ReadStateSpace, P: ReadOwners<Ix = G::Ix>> ReadOwners for BaseModel<G, P> {
    type OwnerType = P::OwnerType;

    derive_read_owners!(players);
}

impl<G: ReadStateSpace, P: ReadPlayers<Ix = G::Ix>> ReadPlayers for BaseModel<G, P> {
    derive_read_players!(players);
}

impl<G: ReadStateSpace, P: ReadOwners<Ix = G::Ix>> BaseModel<G, P> {
    pub fn without_players(self) -> BaseModel<G, SinglePlayer> {
        BaseModel::with_graph(self.graph)
    }
}

impl<G: ReadStateSpace> BaseModel<G, SinglePlayer> {
    pub fn with_players<P: ReadOwners<Ix = G::Ix>>(self, players: P) -> BaseModel<G, P> {
        BaseModel {
            graph: self.graph,
            players,
        }
    }
}

impl<G: BuildStates> BuildStates for BaseModel<G, SinglePlayer> {
    derive_build_states!(graph);
}

impl<G: BuildStates, P: BuildOwners<Ix = G::Ix>> BaseModel<G, P> {
    pub fn add_state(&mut self, owner: P::OwnerType) -> StateIndex<G::Ix> {
        let state = self.graph.add_state();
        self.players.add_state_owner(state, owner);
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

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
{
    pub fn map_base<M2, F: FnOnce(M) -> M2>(
        self,
        map: F,
    ) -> Model<M2, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds> {
        Model {
            base: map(self.base),
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        }
    }

    // On failure, `map` returns the base model, so that the original model can be returned.
    pub fn try_map_base<M2, F: FnOnce(M) -> Result<M2, M>>(
        self,
        map: F,
    ) -> Result<Model<M2, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>, Self> {
        match map(self.base) {
            Ok(base) => Ok(Model {
                base,
                initial: self.initial,
                choice_labels: self.choice_labels,
                branch_labels: self.branch_labels,
                observations: self.observations,
                atomic_propositions: self.atomic_propositions,
                rewards: self.rewards,
                annotations: self.annotations,
                state_valuations: self.state_valuations,
                predecessors: self.predecessors,
            }),
            Err(base) => Err(Model { base, ..self }),
        }
    }
}
