use crate::{ModelBuilder, ModelBuildingError};
use prism_model::{ModelType, Span};
use probabilistic_models::base_model::{
    AnyBaseModel, BaseModelType, Dtmc, Mdp, NonstochasticGame, StochasticGame, TransitionSystem,
};
use probabilistic_models::{HasIx, Ix};
use std::marker::PhantomData;

/// Determines the type of the built model.
///
/// Models are always built as an [`Mdp`], or as a [`StochasticGame`] if the model defines players,
/// and then narrowed to the selected type. Narrowing preserves all indices, so it does not affect
/// the other model components.
pub trait ModelTypeSelection: HasIx {
    type BaseModel;

    /// Called before building. Fails if the selected type cannot be built for a model with or
    /// without players, respectively.
    fn check_players(&self, has_players: bool) -> Result<(), ModelBuildingError>;

    /// Narrows a model without players. `declared` is the model type specified in the PRISM model.
    fn select_without_owners<S: Span>(
        &self,
        declared: Option<&ModelType<S>>,
        mdp: Mdp<Self::Ix>,
    ) -> Result<Self::BaseModel, ModelBuildingError>;

    /// Narrows a model with players. `declared` is the model type specified in the PRISM model.
    fn select_with_owners<S: Span>(
        &self,
        declared: Option<&ModelType<S>>,
        game: StochasticGame<Self::Ix>,
    ) -> Result<Self::BaseModel, ModelBuildingError>;
}

/// Uses the model type specified in the model, or the narrowest possible type if the model does
/// not specify one.
#[derive(Debug, Clone, Copy)]
pub struct Auto<I: Ix> {
    _phantom_data: PhantomData<I>,
}

/// Uses the narrowest possible model type, ignoring the model type specified in the model.
#[derive(Debug, Clone, Copy)]
pub struct IgnoreModelType<I: Ix> {
    _phantom_data: PhantomData<I>,
}

/// Builds exactly the base model type `M` (e.g. [`Dtmc`]), ignoring the model type specified in
/// the model. Building fails if the model does not fit `M`.
#[derive(Debug, Clone, Copy)]
pub struct Fixed<M> {
    _phantom_data: PhantomData<M>,
}

macro_rules! constructors {
    ($selection:ident, $param:ident: $bound:path) => {
        impl<$param: $bound> $selection<$param> {
            pub fn new() -> Self {
                Self {
                    _phantom_data: PhantomData,
                }
            }
        }

        // Not derived, as that would require the type parameter to implement `Default`
        impl<$param: $bound> Default for $selection<$param> {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

constructors!(Auto, I: Ix);
constructors!(IgnoreModelType, I: Ix);
constructors!(Fixed, M: HasIx);

impl<I: Ix> HasIx for Auto<I> {
    type Ix = I;
}

impl<I: Ix> HasIx for IgnoreModelType<I> {
    type Ix = I;
}

impl<M: HasIx> HasIx for Fixed<M> {
    type Ix = M::Ix;
}

fn declared_base_model_type<S: Span>(
    model_type: &ModelType<S>,
) -> Result<BaseModelType, ModelBuildingError> {
    match model_type {
        ModelType::Dtmc(_) => Ok(BaseModelType::Dtmc),
        ModelType::Mdp(_) => Ok(BaseModelType::Mdp),
        ModelType::TransitionSystem(_) => Ok(BaseModelType::TransitionSystem),
        ModelType::TurnBasedStochasticMultiplayerGame(_) => Ok(BaseModelType::StochasticGame),
        ModelType::TurnBasedNonstochasticMultiplayerGame(_) => Ok(BaseModelType::NonstochasticGame),
        ModelType::Ctmc(_) => Err(ModelBuildingError::UnsupportedModelType {
            model_type: model_type.to_string(),
        }),
    }
}

impl<I: Ix> ModelTypeSelection for Auto<I> {
    type BaseModel = AnyBaseModel<I>;

    fn check_players(&self, _has_players: bool) -> Result<(), ModelBuildingError> {
        Ok(())
    }

    fn select_without_owners<S: Span>(
        &self,
        declared: Option<&ModelType<S>>,
        mdp: Mdp<I>,
    ) -> Result<Self::BaseModel, ModelBuildingError> {
        let Some(declared) = declared else {
            return Ok(mdp.narrow());
        };
        let requested = declared_base_model_type(declared)?;
        mdp.try_narrow_to(requested)
            .map_err(|mdp| ModelBuildingError::ModelTypeMismatch {
                requested,
                narrowest: mdp.narrowest_type(),
            })
    }

    fn select_with_owners<S: Span>(
        &self,
        declared: Option<&ModelType<S>>,
        game: StochasticGame<I>,
    ) -> Result<Self::BaseModel, ModelBuildingError> {
        let Some(declared) = declared else {
            return Ok(game.narrow());
        };
        let requested = declared_base_model_type(declared)?;
        game.try_narrow_to(requested)
            .map_err(|game| ModelBuildingError::ModelTypeMismatch {
                requested,
                narrowest: game.narrowest_type(),
            })
    }
}

impl<I: Ix> ModelTypeSelection for IgnoreModelType<I> {
    type BaseModel = AnyBaseModel<I>;

    fn check_players(&self, _has_players: bool) -> Result<(), ModelBuildingError> {
        Ok(())
    }

    fn select_without_owners<S: Span>(
        &self,
        _declared: Option<&ModelType<S>>,
        mdp: Mdp<I>,
    ) -> Result<Self::BaseModel, ModelBuildingError> {
        Ok(mdp.narrow())
    }

    fn select_with_owners<S: Span>(
        &self,
        _declared: Option<&ModelType<S>>,
        game: StochasticGame<I>,
    ) -> Result<Self::BaseModel, ModelBuildingError> {
        Ok(game.narrow())
    }
}

macro_rules! fixed_without_owners {
    ($model:ident, |$mdp:ident| $narrow:expr) => {
        impl<I: Ix> ModelTypeSelection for Fixed<$model<I>> {
            type BaseModel = $model<I>;

            fn check_players(&self, has_players: bool) -> Result<(), ModelBuildingError> {
                match has_players {
                    true => Err(ModelBuildingError::ModelHasPlayers),
                    false => Ok(()),
                }
            }

            fn select_without_owners<S: Span>(
                &self,
                _declared: Option<&ModelType<S>>,
                $mdp: Mdp<I>,
            ) -> Result<Self::BaseModel, ModelBuildingError> {
                let narrowed: Result<$model<I>, Mdp<I>> = $narrow;
                narrowed.map_err(|mdp| ModelBuildingError::ModelTypeMismatch {
                    requested: BaseModelType::$model,
                    narrowest: mdp.narrowest_type(),
                })
            }

            fn select_with_owners<S: Span>(
                &self,
                _declared: Option<&ModelType<S>>,
                _game: StochasticGame<I>,
            ) -> Result<Self::BaseModel, ModelBuildingError> {
                unreachable!("`check_players` already returns an error for a model with players")
            }
        }
    };
}

macro_rules! fixed_with_owners {
    ($model:ident, |$game:ident| $narrow:expr) => {
        impl<I: Ix> ModelTypeSelection for Fixed<$model<I>> {
            type BaseModel = $model<I>;

            fn check_players(&self, has_players: bool) -> Result<(), ModelBuildingError> {
                match has_players {
                    true => Ok(()),
                    false => Err(ModelBuildingError::ModelHasNoPlayers),
                }
            }

            fn select_without_owners<S: Span>(
                &self,
                _declared: Option<&ModelType<S>>,
                _mdp: Mdp<I>,
            ) -> Result<Self::BaseModel, ModelBuildingError> {
                unreachable!("`check_players` already returns an error for a model without players")
            }

            fn select_with_owners<S: Span>(
                &self,
                _declared: Option<&ModelType<S>>,
                $game: StochasticGame<I>,
            ) -> Result<Self::BaseModel, ModelBuildingError> {
                let narrowed: Result<$model<I>, StochasticGame<I>> = $narrow;
                narrowed.map_err(|game| ModelBuildingError::ModelTypeMismatch {
                    requested: BaseModelType::$model,
                    narrowest: game.narrowest_type(),
                })
            }
        }
    };
}

fixed_without_owners!(Mdp, |mdp| Ok(mdp));
fixed_without_owners!(Dtmc, |mdp| mdp.try_into_dtmc());
fixed_without_owners!(TransitionSystem, |mdp| mdp.try_into_transition_system());
fixed_with_owners!(StochasticGame, |game| Ok(game));
fixed_with_owners!(NonstochasticGame, |game| game.try_into_nonstochastic_game());

impl<
    'a,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    Sel: ModelTypeSelection,
    IB: crate::initial_states_builder::InitialStatesBuilder<Ix = Sel::Ix>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<Ix = Sel::Ix>,
    CL: crate::choice_labels::ChoiceLabelBuilder<Ix = Sel::Ix>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = Sel::Ix>,
> ModelBuilder<'a, S, Q, L, IS, Sel, IB, APs, CL, Rew>
{
    /// Selects the type of the built model, see [`ModelTypeSelection`].
    pub fn with_model_type_selection<Sel2: ModelTypeSelection<Ix = Sel::Ix>>(
        self,
        selection: Sel2,
    ) -> ModelBuilder<'a, S, Q, L, IS, Sel2, IB, APs, CL, Rew> {
        self.map_selection(selection)
    }

    /// Uses the model type specified in the model, or the narrowest possible type if the model
    /// does not specify one. The built model is an [`AnyBaseModel`].
    pub fn with_auto_model_type(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, Auto<Sel::Ix>, IB, APs, CL, Rew> {
        self.with_model_type_selection(Auto::new())
    }

    /// Uses the narrowest possible model type, ignoring the model type specified in the model.
    /// The built model is an [`AnyBaseModel`].
    pub fn with_narrowest_model_type(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, IgnoreModelType<Sel::Ix>, IB, APs, CL, Rew> {
        self.with_model_type_selection(IgnoreModelType::new())
    }

    /// Builds exactly the base model type `M` (e.g. `Dtmc<I>`), ignoring the model type specified
    /// in the model.
    pub fn with_fixed_model_type<M: HasIx<Ix = Sel::Ix>>(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, Fixed<M>, IB, APs, CL, Rew>
    where
        Fixed<M>: ModelTypeSelection<Ix = Sel::Ix>,
    {
        self.with_model_type_selection(Fixed::new())
    }

    /// Builds an [`Mdp`], ignoring the model type specified in the model. Shorthand for
    /// [`with_fixed_model_type::<Mdp<_>>()`](Self::with_fixed_model_type).
    pub fn with_mdp_model_type(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, Fixed<Mdp<Sel::Ix>>, IB, APs, CL, Rew> {
        self.with_fixed_model_type::<Mdp<Sel::Ix>>()
    }

    /// Builds a [`Dtmc`], ignoring the model type specified in the model. Shorthand for
    /// [`with_fixed_model_type::<Dtmc<_>>()`](Self::with_fixed_model_type).
    pub fn with_dtmc_model_type(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, Fixed<Dtmc<Sel::Ix>>, IB, APs, CL, Rew> {
        self.with_fixed_model_type::<Dtmc<Sel::Ix>>()
    }

    /// Builds a [`TransitionSystem`], ignoring the model type specified in the model. Shorthand
    /// for [`with_fixed_model_type::<TransitionSystem<_>>()`](Self::with_fixed_model_type).
    pub fn with_transition_system_model_type(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, Fixed<TransitionSystem<Sel::Ix>>, IB, APs, CL, Rew> {
        self.with_fixed_model_type::<TransitionSystem<Sel::Ix>>()
    }

    /// Builds a [`StochasticGame`], ignoring the model type specified in the model. Shorthand
    /// for [`with_fixed_model_type::<StochasticGame<_>>()`](Self::with_fixed_model_type).
    pub fn with_stochastic_game_model_type(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, Fixed<StochasticGame<Sel::Ix>>, IB, APs, CL, Rew> {
        self.with_fixed_model_type::<StochasticGame<Sel::Ix>>()
    }

    /// Builds a [`NonstochasticGame`], ignoring the model type specified in the model. Shorthand
    /// for [`with_fixed_model_type::<NonstochasticGame<_>>()`](Self::with_fixed_model_type).
    pub fn with_nonstochastic_game_model_type(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, Fixed<NonstochasticGame<Sel::Ix>>, IB, APs, CL, Rew> {
        self.with_fixed_model_type::<NonstochasticGame<Sel::Ix>>()
    }
}
