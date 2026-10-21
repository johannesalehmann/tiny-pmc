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
