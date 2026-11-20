use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

/// An empty marker type. This should be used in conjunction with non-game modele: They don't
/// contain players, so coalitions don't make sense for them.
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct NoCoalition;

impl Display for NoCoalition {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

/// A coalition of players, contain both named players and players referred to by their index.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Coalition {
    /// The players in the coalition that are referred to by name
    pub names: BTreeSet<String>,
    /// The players in the coalition that are referenced by their 1-based index.
    pub indices: BTreeSet<usize>,
}

impl Coalition {
    /// Constructs a coalition with the given player names and (1-based) player indices.
    pub fn new(names: BTreeSet<String>, indices: BTreeSet<usize>) -> Self {
        Self { names, indices }
    }
}

impl Display for Coalition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<<")?;
        let names = self.names.iter().map(|name| name.to_string());
        let indices = self.indices.iter().map(|index| index.to_string());
        for (i, player) in names.chain(indices).enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{player}")?;
        }
        write!(f, ">>")
    }
}

/// Transforms a coalition into a reference to itself.
pub trait CoalitionAsRef {
    /// The type of the coalition by reference
    type Output<'a>
    where
        Self: 'a;

    /// Returns the coalition by reference.
    fn as_ref(&self) -> Self::Output<'_>;
}

/// Transforms a coalition into a mutable reference to itself
pub trait CoalitionAsMut {
    /// The type of the coalition by mutable reference
    type Output<'a>
    where
        Self: 'a;

    /// Returns the coalition by mutable reference.
    fn as_mut(&mut self) -> Self::Output<'_>;
}

impl CoalitionAsRef for NoCoalition {
    type Output<'a> = NoCoalition;

    fn as_ref(&self) -> Self::Output<'_> {
        NoCoalition
    }
}

impl CoalitionAsMut for NoCoalition {
    type Output<'a> = NoCoalition;

    fn as_mut(&mut self) -> Self::Output<'_> {
        NoCoalition
    }
}

impl CoalitionAsRef for Coalition {
    type Output<'a> = &'a Coalition;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl CoalitionAsMut for Coalition {
    type Output<'a> = &'a mut Coalition;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}
