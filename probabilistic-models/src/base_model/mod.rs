use crate::traits::ReadStateSpace;
use typed_index_collections::Index;

mod mdp;
mod nonstochastic_game;
mod stochastic_game;
mod transition_system;

pub use mdp::*;

pub trait BaseModel: ReadStateSpace {}
