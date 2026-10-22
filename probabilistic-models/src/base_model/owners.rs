use crate::traits::ClearModel;
use crate::{HasIx, Ix, Model, PlayerIndex, StateIndex};
use typed_index_collections::{Index, NamedTo1, RawIndex, SemiboundedIndexRange, To1};

pub trait ReadOwners: HasIx {
    type OwnerType;
    fn state_owner(&self, state: StateIndex<Self::Ix>) -> Self::OwnerType;
}

macro_rules! derive_read_owners {
    ($subcomponent:ident) => {
        fn state_owner(&self, state: $crate::StateIndex<Self::Ix>) -> Self::OwnerType {
            self.$subcomponent.state_owner(state)
        }
    };
}
pub(crate) use derive_read_owners;

impl<M: ReadOwners, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> ReadOwners
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type OwnerType = M::OwnerType;

    derive_read_owners!(base);
}

impl<T: ReadOwners> ReadOwners for &T {
    type OwnerType = T::OwnerType;

    fn state_owner(&self, state: StateIndex<Self::Ix>) -> Self::OwnerType {
        (**self).state_owner(state)
    }
}
impl<T: ReadOwners> ReadOwners for &mut T {
    type OwnerType = T::OwnerType;

    fn state_owner(&self, state: StateIndex<Self::Ix>) -> Self::OwnerType {
        (**self).state_owner(state)
    }
}

pub trait BuildOwners: ReadOwners {
    // `state` must be the next state index, i.e. the number of states that already have an owner.
    fn add_state_owner(&mut self, state: StateIndex<Self::Ix>, owner: Self::OwnerType);
}

impl<T: BuildOwners> BuildOwners for &mut T {
    fn add_state_owner(&mut self, state: StateIndex<Self::Ix>, owner: Self::OwnerType) {
        (**self).add_state_owner(state, owner)
    }
}

// TODO: It would be nice to just require M: SinglePlayer (avoiding the name clash in some way),
//  instead of M: ReadOwners<OwnerType = SinglePlayer>
// Owner of every state in a model without multiple players, and also the players component of such
// a model. Algorithms can require M: ReadOwners<OwnerType = SinglePlayer> to ensure that no model
// with owners is passed in (which the algorithm might not be able to handle).
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct SinglePlayer;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TwoPlayer {
    Eve,
    Adam,
}

/// Provides the named players of a game.
pub trait ReadPlayers: ReadOwners<OwnerType = PlayerIndex<<Self as HasIx>::Ix>> {
    fn players(&self) -> SemiboundedIndexRange<PlayerIndex<Self::Ix>>;
    fn player_name(&self, player: PlayerIndex<Self::Ix>) -> &str;
    fn player_index(&self, name: &str) -> Option<PlayerIndex<Self::Ix>>;
}

macro_rules! derive_read_players {
    ($subcomponent:ident) => {
        fn players(
            &self,
        ) -> typed_index_collections::SemiboundedIndexRange<$crate::PlayerIndex<Self::Ix>> {
            self.$subcomponent.players()
        }

        fn player_name(&self, player: $crate::PlayerIndex<Self::Ix>) -> &str {
            self.$subcomponent.player_name(player)
        }

        fn player_index(&self, name: &str) -> Option<$crate::PlayerIndex<Self::Ix>> {
            self.$subcomponent.player_index(name)
        }
    };
}
pub(crate) use derive_read_players;

impl<M: ReadPlayers, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> ReadPlayers
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    derive_read_players!(base);
}

impl<T: ReadPlayers> ReadPlayers for &T {
    fn players(&self) -> SemiboundedIndexRange<PlayerIndex<Self::Ix>> {
        (**self).players()
    }

    fn player_name(&self, player: PlayerIndex<Self::Ix>) -> &str {
        (**self).player_name(player)
    }

    fn player_index(&self, name: &str) -> Option<PlayerIndex<Self::Ix>> {
        (**self).player_index(name)
    }
}

impl<T: ReadPlayers> ReadPlayers for &mut T {
    fn players(&self) -> SemiboundedIndexRange<PlayerIndex<Self::Ix>> {
        (**self).players()
    }

    fn player_name(&self, player: PlayerIndex<Self::Ix>) -> &str {
        (**self).player_name(player)
    }

    fn player_index(&self, name: &str) -> Option<PlayerIndex<Self::Ix>> {
        (**self).player_index(name)
    }
}

/// Owners of a turn-based game with named players, where each state is owned by exactly one
/// player.
#[derive(Debug, Clone)]
pub struct TurnBased<I: Ix> {
    pub state_to_owner: To1<StateIndex<I>, PlayerIndex<I>>,
    pub players: NamedTo1<PlayerIndex<I>, ()>,
}

impl<I: Ix> HasIx for TurnBased<I> {
    type Ix = I;
}

impl<I: Ix> Default for TurnBased<I> {
    fn default() -> Self {
        Self {
            state_to_owner: To1::new(),
            players: NamedTo1::new(),
        }
    }
}

// Not derived, as `NamedTo1` compares names independent of their order, whereas the order of
// players matters here, as it determines the player indices referenced by `state_to_owner`.
impl<I: Ix> PartialEq for TurnBased<I> {
    fn eq(&self, other: &Self) -> bool {
        self.players.names() == other.players.names() && self.state_to_owner == other.state_to_owner
    }
}

impl<I: Ix> TurnBased<I> {
    pub fn with_players(names: impl IntoIterator<Item = String>) -> Self {
        let mut owners = Self::default();
        for name in names {
            owners.add_player(name);
        }
        owners
    }

    // Panics if a player with the same name already exists.
    pub fn add_player(&mut self, name: String) -> PlayerIndex<I> {
        self.players.add_entry(name, ())
    }
}

impl<I: Ix> ReadPlayers for TurnBased<I> {
    fn players(&self) -> SemiboundedIndexRange<PlayerIndex<Self::Ix>> {
        self.players.internal_indices()
    }

    fn player_name(&self, player: PlayerIndex<Self::Ix>) -> &str {
        &self.players.names()[player]
    }

    fn player_index(&self, name: &str) -> Option<PlayerIndex<Self::Ix>> {
        self.players.index_by_name(name)
    }
}

impl<I: Ix> ReadOwners for TurnBased<I> {
    type OwnerType = PlayerIndex<I>;

    fn state_owner(&self, state: StateIndex<Self::Ix>) -> Self::OwnerType {
        self.state_to_owner[state]
    }
}

impl<I: Ix> BuildOwners for TurnBased<I> {
    fn add_state_owner(&mut self, state: StateIndex<Self::Ix>, owner: Self::OwnerType) {
        debug_assert!(
            owner.raw().as_usize() < self.players.len(),
            "{owner:?} is not a player of this game, which has {} players",
            self.players.len()
        );
        // TODO: On a mismatch, `add_checked` panics with a bare `left == right` assertion that does
        //  not mention state owners. Provide a more helpful message.
        self.state_to_owner.add_checked(state, owner);
    }
}

impl ClearModel for SinglePlayer {
    fn clear(&mut self) {}
}

// Removes all state owners, but keeps the players.
impl<I: Ix> ClearModel for TurnBased<I> {
    fn clear(&mut self) {
        self.state_to_owner.clear();
    }
}
