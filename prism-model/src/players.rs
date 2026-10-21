use crate::{FullSpan, Identifier, Span};
use std::fmt::{Display, Formatter};

/// A player struct, consisting of a name and a list of controlled modules and actions.
///
/// This corresponds to the PRISM-games syntax, where `p1` is the name, `M1` and `M2` are the
/// controlled modules and `a` and `b` are the controlled actions.
///
/// ```prism
/// player p1
///     M1, M2, [a], [b]
/// endplayer
/// ```
///
/// The semantics follow PRISM-Games, i.e. a choice is controlled if it is either labelled by
/// controlled action or belongs to a controlled module. A state is controlled by a player if the
/// player controls at least one choice of the state.
///
/// Only one player must control transitions for any given state. It is OK if some choices are not
/// controlled, as long as at least one choice is controlled for each state.
///
/// Players are stored in [`Model::players`](crate::Model::players).
#[derive(PartialEq, Clone, Debug)]
pub struct Player<S: Span = FullSpan> {
    /// The [`Span`] of the entire player definition block
    pub span: S,
    /// The name of the player
    pub name: Identifier<S>,
    /// The modules controlled by this player.
    pub controlled_modules: Vec<Identifier<S>>,
    /// The actions controlled by this player.
    pub controlled_actions: Vec<Identifier<S>>,
}

impl<S: Span> Player<S> {
    /// Constructs a `Player` with given name, controlled modules and controlled actions and empty
    /// [`Span`].
    ///
    /// To construct a `Player` with given span, use [`Player::new_spanned()`].
    pub fn new(
        name: Identifier<S>,
        controlled_modules: Vec<Identifier<S>>,
        controlled_actions: Vec<Identifier<S>>,
    ) -> Self {
        Self::new_spanned(name, controlled_modules, controlled_actions, S::empty())
    }

    /// Constructs a `Player` with given name, controlled modules, controlled actions and [`Span`].
    ///
    /// To construct a `Player` with empty span, use [`Player::new()`].
    pub fn new_spanned(
        name: Identifier<S>,
        controlled_modules: Vec<Identifier<S>>,
        controlled_actions: Vec<Identifier<S>>,
        span: S,
    ) -> Self {
        Self {
            span,
            name,
            controlled_modules,
            controlled_actions,
        }
    }

    /// Maps every [`Span`] of this `Player` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to the name, every controlled module and action and the span of the
    /// player.
    ///
    /// Usage is analogous to [`Expression::map_span()`](crate::Expression::map_span). Refer to its
    /// documentation for details and examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Player<S2> {
        Player {
            span: map(self.span),
            name: self.name.map_span(map),
            controlled_modules: self
                .controlled_modules
                .into_iter()
                .map(|m| m.map_span(map))
                .collect(),
            controlled_actions: self
                .controlled_actions
                .into_iter()
                .map(|a| a.map_span(map))
                .collect(),
        }
    }
}

impl<S: Span> Display for Player<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "player {}", self.name)?;
        if !self.controlled_modules.is_empty() || !self.controlled_actions.is_empty() {
            write!(f, "    ")?;
            let mut is_first = true;
            for module in &self.controlled_modules {
                if !is_first {
                    write!(f, ", ")?;
                }
                is_first = false;
                write!(f, "{}", module)?;
            }
            for action in &self.controlled_actions {
                if !is_first {
                    write!(f, ", ")?;
                }
                is_first = false;
                write!(f, "[{}]", action)?;
            }
            writeln!(f)?;
        }
        write!(f, "endplayer")
    }
}

/// A collection of players. Each player has a unique name.
///
/// Players are stored in [`Model::players`](crate::Model::players).
#[derive(PartialEq, Clone, Debug)]
pub struct PlayerManager<S: Span = FullSpan> {
    /// The set of players.
    ///
    /// Instead of adding players to this directly, use [`PlayerManager::add()`], which ensures
    /// there are no duplicates.
    players: Vec<Player<S>>,
}

impl<S: Span> PlayerManager<S> {
    /// Creates an empty `PlayerManager`.
    ///
    /// To add players, use [`PlayerManager::add()`] or create the `PlayerManager` with
    /// [`PlayerManager::with_players()`].
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
        }
    }

    /// Creates a `PlayerManager` with the given players.
    ///
    /// If the set of players contains duplicates, [`PlayerExists`] is returned. `index: 1`
    /// indicates that the first occurrence of the duplicate player was at index `1`.
    ///
    /// To create an empty `PlayerManager`, use [`PlayerManager::new()`] and add players with
    /// [`PlayerManager::add()`]. To skip the duplicate check, use
    /// [`PlayerManager::with_players_unchecked()`].
    pub fn with_players(mut players: Vec<Player<S>>) -> Result<Self, PlayerExists> {
        let mut res = Self::new();

        for player in players.drain(..) {
            res.add(player)?;
        }
        Ok(res)
    }

    /// Creates a `PlayerManager` with the given players, *without checking for duplicate names*.
    ///
    /// This is much faster than [`PlayerManager::with_players()`], but only suitable if you know
    /// that `players` contains no players with duplicate names.
    pub fn with_players_unchecked(players: Vec<Player<S>>) -> Self {
        Self { players }
    }

    /// Returns the player with the given index or `None` if the index is out of bounds.
    pub fn get(&self, index: usize) -> Option<&Player<S>> {
        self.players.get(index)
    }

    /// Adds a new player to this `PlayerManager`.
    ///
    /// If a player with the same name already exists, returns [`PlayerExists`].
    pub fn add(&mut self, player: Player<S>) -> Result<(), PlayerExists> {
        for (index, other_player) in self.players.iter().enumerate() {
            if other_player.name == player.name {
                return Err(PlayerExists { index });
            }
        }
        self.players.push(player);
        Ok(())
    }

    /// Removes all players from this `PlayerManager`.
    pub fn clear(&mut self) {
        self.players.clear();
    }

    /// Returns `true` if the `PlayerManager` contains no players, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    /// Returns the number of players contained in this `PlayerManager`.
    pub fn len(&self) -> usize {
        self.players.len()
    }

    /// Creates an iterator (by reference) over the players in this `PlayerManager`.
    pub fn iter(&self) -> impl Iterator<Item = &Player<S>> {
        self.into_iter()
    }

    /// Creates an iterator (by mutable reference) over the players in this `PlayerManager`.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Player<S>> {
        self.into_iter()
    }

    /// Maps the [`Span`] of every [`Player`] in this `PlayerManager` according to mapping function
    /// `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// Usage is analogous to [`Player::map_span()`]. Refer to its documentation for details.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> PlayerManager<S2> {
        PlayerManager {
            players: self.players.into_iter().map(|p| p.map_span(map)).collect(),
        }
    }
}

impl<'a, S: Span> IntoIterator for &'a PlayerManager<S> {
    type Item = &'a Player<S>;
    type IntoIter = std::slice::Iter<'a, Player<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.players.iter()
    }
}

impl<'a, S: Span> IntoIterator for &'a mut PlayerManager<S> {
    type Item = &'a mut Player<S>;
    type IntoIter = std::slice::IterMut<'a, Player<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.players.iter_mut()
    }
}

impl<S: Span> IntoIterator for PlayerManager<S> {
    type Item = Player<S>;
    type IntoIter = std::vec::IntoIter<Player<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.players.into_iter()
    }
}

/// Error produced while adding a player to a [`PlayerManager`], indicating that a player with this
/// name already exists.
#[derive(Debug)]
pub struct PlayerExists {
    /// The index of the existing player with the same name.
    ///
    /// Use [`PlayerManager::get(index)`](PlayerManager::get()) to retrieve details about the
    /// existing player.
    pub index: usize,
}
