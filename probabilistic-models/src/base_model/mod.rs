use crate::traits::ReadStateSpace;

mod mdp;
pub use mdp::{Mdp, StateChoiceBranchTriples, StateChoiceBranchTriplesIterator, StateChoicePairs};

mod nonstochastic_game;
pub use nonstochastic_game::NonstochasticGame;

mod stochastic_game;
pub use stochastic_game::TwoPlayerTurnBasedGame;

mod transition_system;
pub use transition_system::TransitionSystem;

pub trait BaseModel: ReadStateSpace {}
