use crate::ModelBuildingError;
use crate::synchronised_actions::SynchronisedActions;
use prism_model::{Identifier, Model, Span, VariableReference};
use probabilistic_models::base_model::{BuildOwners, SinglePlayer, TurnBased, TwoPlayer};
use probabilistic_models::{HasIx, Ix, StateIndex};
use std::marker::PhantomData;
use typed_index_collections::{Index, RawIndex};

/// Determines which player controls which choice, according to the player definitions of a model.
///
/// A choice is controlled by a player if its action is listed in the player's controlled actions,
/// or if the action is internal to a single module (i.e. it is unlabelled or does not synchronise
/// with any other module) and that module is controlled by the player. Choices of synchronising
/// actions that are not listed by any player are uncontrolled.
pub(crate) struct PlayerControl {
    player_names: Vec<String>,
    unsynchronised_owners: Vec<Option<usize>>,
    synchronised_owners: Vec<Option<usize>>,
}

impl PlayerControl {
    pub(crate) fn new<S: Span, E>(
        model: &Model<VariableReference, S, E, Identifier<S>>,
        synchronised_actions: &SynchronisedActions,
    ) -> Result<Self, ModelBuildingError> {
        let player_names: Vec<String> = model.players.iter().map(|p| p.name.name.clone()).collect();

        let mut module_owners = vec![None; model.modules.len()];
        let mut synchronised_owners = vec![None; synchronised_actions.len()];
        for (player_index, player) in model.players.iter().enumerate() {
            for module in &player.controlled_modules {
                let module_index =
                    model
                        .modules
                        .get_index_by_str(&module.name)
                        .ok_or_else(|| ModelBuildingError::UnknownPlayerModule {
                            player: player.name.name.clone(),
                            module: module.name.clone(),
                        })?;
                Self::set_owner(&mut module_owners[module_index], player_index).map_err(
                    |other| ModelBuildingError::ModuleControlledByMultiplePlayers {
                        module: module.name.clone(),
                        players: (player_names[other].clone(), player.name.name.clone()),
                    },
                )?;
            }
            for action in &player.controlled_actions {
                let action_index =
                    synchronised_actions.index_of(&action.name).ok_or_else(|| {
                        ModelBuildingError::UnknownPlayerAction {
                            player: player.name.name.clone(),
                            action: action.name.clone(),
                        }
                    })?;
                Self::set_owner(&mut synchronised_owners[action_index], player_index).map_err(
                    |other| ModelBuildingError::ActionControlledByMultiplePlayers {
                        action: action.name.clone(),
                        players: (player_names[other].clone(), player.name.name.clone()),
                    },
                )?;
            }
        }

        // Labelled actions that occur in only a single module are internal to that module.
        for (action_index, action) in synchronised_actions.into_iter().enumerate() {
            if let [module] = action.participating_modules.as_slice()
                && let Some(module_owner) = module_owners[module.module_index]
            {
                Self::set_owner(&mut synchronised_owners[action_index], module_owner).map_err(
                    |other| ModelBuildingError::ActionControlledByMultiplePlayers {
                        action: action.name.clone(),
                        players: (
                            player_names[other].clone(),
                            player_names[module_owner].clone(),
                        ),
                    },
                )?;
            }
        }

        Ok(Self {
            player_names,
            unsynchronised_owners: module_owners,
            synchronised_owners,
        })
    }

    // On a conflict, returns the existing owner.
    fn set_owner(owner: &mut Option<usize>, player_index: usize) -> Result<(), usize> {
        match *owner {
            Some(existing) if existing != player_index => Err(existing),
            _ => {
                *owner = Some(player_index);
                Ok(())
            }
        }
    }

    pub fn player_count(&self) -> usize {
        self.player_names.len()
    }

    pub fn player_name(&self, player_index: usize) -> &str {
        &self.player_names[player_index]
    }

    pub fn unsynchronised_owner(&self, module_index: usize) -> Option<usize> {
        self.unsynchronised_owners[module_index]
    }

    pub fn synchronised_owner(&self, synchronised_action_index: usize) -> Option<usize> {
        self.synchronised_owners[synchronised_action_index]
    }
}

/// Tracks the owner of each state while the model is built.
///
/// For every state, the state builder reports each added choice (except choices added to fix
/// deadlocks) and then calls [`OwnersBuilder::finish_state()`]. States are finished in order of
/// their indices.
pub(crate) trait OwnersBuilder: HasIx {
    type Owners;

    fn unsynchronised_choice_added(&mut self, module_index: usize);
    fn synchronised_choice_added(&mut self, synchronised_action_index: usize);
    fn finish_state(&mut self, state: StateIndex<Self::Ix>) -> Result<(), ModelBuildingError>;
    fn into_owners(self) -> Self::Owners;
}

/// Used for models without players.
pub(crate) struct NoOwners<I: Ix> {
    _phantom_data: PhantomData<I>,
}

impl<I: Ix> HasIx for NoOwners<I> {
    type Ix = I;
}

impl<I: Ix> NoOwners<I> {
    pub fn new() -> Self {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

impl<I: Ix> OwnersBuilder for NoOwners<I> {
    type Owners = SinglePlayer;

    fn unsynchronised_choice_added(&mut self, _module_index: usize) {}

    fn synchronised_choice_added(&mut self, _synchronised_action_index: usize) {}

    fn finish_state(&mut self, _state: StateIndex<I>) -> Result<(), ModelBuildingError> {
        Ok(())
    }

    fn into_owners(self) -> Self::Owners {
        SinglePlayer
    }
}

/// Builds turn-based models, where each state is owned by exactly one player.
///
/// Building fails if a state has choices, but none of them is controlled. States without choices
/// (deadlock states and states excluded by the state space restriction) are owned by the first
/// player.
pub(crate) struct TurnBasedOwnersBuilder<I: Ix> {
    control: PlayerControl,
    owners: TurnBased<I, TwoPlayer>,
    // The owner of the current state, if any of its choices so far is controlled
    current_owner: Option<usize>,
    // A second player controlling a choice of the current state, if any
    conflicting_owner: Option<usize>,
    // Whether any choice (controlled or not) was added to the current state
    has_choices: bool,
}

impl<I: Ix> HasIx for TurnBasedOwnersBuilder<I> {
    type Ix = I;
}

impl<I: Ix> TurnBasedOwnersBuilder<I> {
    pub fn new(control: PlayerControl) -> Self {
        // TODO: Support more than two players
        if control.player_count() > 2 {
            panic!(
                "Models with more than two players are not yet supported, the model has {} players",
                control.player_count()
            );
        }
        Self {
            control,
            owners: TurnBased::default(),
            current_owner: None,
            conflicting_owner: None,
            has_choices: false,
        }
    }

    fn choice_added(&mut self, owner: Option<usize>) {
        self.has_choices = true;
        match (self.current_owner, owner) {
            (_, None) => {}
            (None, Some(_)) => self.current_owner = owner,
            (Some(current), Some(new)) => {
                if current != new {
                    self.conflicting_owner = Some(new);
                }
            }
        }
    }
}

impl<I: Ix> OwnersBuilder for TurnBasedOwnersBuilder<I> {
    type Owners = TurnBased<I, TwoPlayer>;

    fn unsynchronised_choice_added(&mut self, module_index: usize) {
        let owner = self.control.unsynchronised_owner(module_index);
        self.choice_added(owner);
    }

    fn synchronised_choice_added(&mut self, synchronised_action_index: usize) {
        let owner = self.control.synchronised_owner(synchronised_action_index);
        self.choice_added(owner);
    }

    fn finish_state(&mut self, state: StateIndex<I>) -> Result<(), ModelBuildingError> {
        let has_choices = std::mem::take(&mut self.has_choices);
        let owner = match self.current_owner.take() {
            Some(owner) => owner,
            None if !has_choices => 0,
            None => {
                return Err(ModelBuildingError::StateWithoutOwner {
                    state: state.raw().as_usize(),
                });
            }
        };
        if let Some(other) = self.conflicting_owner.take() {
            return Err(ModelBuildingError::StateControlledByMultiplePlayers {
                state: state.raw().as_usize(),
                players: (
                    self.control.player_name(owner).to_string(),
                    self.control.player_name(other).to_string(),
                ),
            });
        }
        let owner = match owner {
            0 => TwoPlayer::Eve,
            _ => TwoPlayer::Adam,
        };
        self.owners.add_state_owner(state, owner);
        Ok(())
    }

    fn into_owners(self) -> Self::Owners {
        self.owners
    }
}
