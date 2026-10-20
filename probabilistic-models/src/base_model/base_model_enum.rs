use crate::base_model::{
    BaseModel, BranchGraph, ChoiceBranchGraph, ChoiceGraph, Dtmc, Mdp, NonstochasticGame,
    SinglePlayer, StochasticGame, TransitionSystem, TurnBased, TwoPlayer,
};
use crate::{DefaultIx, HasIx, Ix, Model};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BaseModelEnum<TS, D, M, NG, SG> {
    TransitionSystem(TS),
    Dtmc(D),
    Mdp(M),
    NonstochasticGame(NG),
    StochasticGame(SG),
}

pub type AnyBaseModel<I = DefaultIx> =
    BaseModelEnum<TransitionSystem<I>, Dtmc<I>, Mdp<I>, NonstochasticGame<I>, StochasticGame<I>>;

impl<TS, D, M, NG, SG> HasIx for BaseModelEnum<TS, D, M, NG, SG>
where
    TS: HasIx,
    D: HasIx<Ix = TS::Ix>,
    M: HasIx<Ix = TS::Ix>,
    NG: HasIx<Ix = TS::Ix>,
    SG: HasIx<Ix = TS::Ix>,
{
    type Ix = TS::Ix;
}

macro_rules! variant_accessors {
    ($variant:ident, $ty:ident, $is:ident, $into:ident, $try_into:ident) => {
        pub fn $is(&self) -> bool {
            matches!(self, BaseModelEnum::$variant(_))
        }

        pub fn $into(self) -> Option<$ty> {
            self.$try_into().ok()
        }

        // Like the `into_...` function, but returns the model on a mismatch, so that other
        // variants can be tried without losing it.
        pub fn $try_into(self) -> Result<$ty, Self> {
            match self {
                BaseModelEnum::$variant(m) => Ok(m),
                other => Err(other),
            }
        }
    };
}

impl<TS, D, M, NG, SG> BaseModelEnum<TS, D, M, NG, SG> {
    variant_accessors!(
        TransitionSystem,
        TS,
        is_transition_system,
        into_transition_system,
        try_into_transition_system
    );
    variant_accessors!(Dtmc, D, is_dtmc, into_dtmc, try_into_dtmc);
    variant_accessors!(Mdp, M, is_mdp, into_mdp, try_into_mdp);
    variant_accessors!(
        NonstochasticGame,
        NG,
        is_nonstochastic_game,
        into_nonstochastic_game,
        try_into_nonstochastic_game
    );
    variant_accessors!(
        StochasticGame,
        SG,
        is_stochastic_game,
        into_stochastic_game,
        try_into_stochastic_game
    );
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
{
    pub fn with_base_model_enum<TS, D, M2, NG, SG>(
        self,
    ) -> Model<
        BaseModelEnum<TS, D, M2, NG, SG>,
        Ini,
        ChLabel,
        BrLabel,
        Obs,
        APs,
        Rew,
        Ann,
        Val,
        Preds,
    >
    where
        M: Into<BaseModelEnum<TS, D, M2, NG, SG>>,
    {
        self.map_base(|m| m.into())
    }
}

macro_rules! model_variant_accessors {
    (
        $ty:ident,
        $is:ident,
        $try_into:ident,
        $base_is:ident,
        $base_into:ident,
        $base_try_into:ident
    ) => {
        pub fn $base_is(&self) -> bool {
            self.base.$is()
        }

        pub fn $base_into(
            self,
        ) -> Option<Model<$ty, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>> {
            self.$base_try_into().ok()
        }

        pub fn $base_try_into(
            self,
        ) -> Result<Model<$ty, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>, Self> {
            self.try_map_base(|m| m.$try_into())
        }
    };
}

impl<TS, D, M, NG, SG, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    Model<BaseModelEnum<TS, D, M, NG, SG>, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
{
    model_variant_accessors!(
        TS,
        is_transition_system,
        try_into_transition_system,
        base_is_transition_system,
        base_into_transition_system,
        base_try_into_transition_system
    );
    model_variant_accessors!(
        D,
        is_dtmc,
        try_into_dtmc,
        base_is_dtmc,
        base_into_dtmc,
        base_try_into_dtmc
    );
    model_variant_accessors!(
        M,
        is_mdp,
        try_into_mdp,
        base_is_mdp,
        base_into_mdp,
        base_try_into_mdp
    );
    model_variant_accessors!(
        NG,
        is_nonstochastic_game,
        try_into_nonstochastic_game,
        base_is_nonstochastic_game,
        base_into_nonstochastic_game,
        base_try_into_nonstochastic_game
    );
    model_variant_accessors!(
        SG,
        is_stochastic_game,
        try_into_stochastic_game,
        base_is_stochastic_game,
        base_into_stochastic_game,
        base_try_into_stochastic_game
    );
}

macro_rules! from_impls {
    ($variant:ident, $graph:ident, $players:ident) => {
        impl<I: Ix> From<BaseModel<$graph<I>, from_impls!(@owned $players, I)>>
            for BaseModelEnum<
                BaseModel<ChoiceGraph<I>, SinglePlayer>,
                BaseModel<BranchGraph<I>, SinglePlayer>,
                BaseModel<ChoiceBranchGraph<I>, SinglePlayer>,
                BaseModel<ChoiceGraph<I>, TurnBased<I, TwoPlayer>>,
                BaseModel<ChoiceBranchGraph<I>, TurnBased<I, TwoPlayer>>,
            >
        {
            fn from(value: BaseModel<$graph<I>, from_impls!(@owned $players, I)>) -> Self {
                Self::$variant(value)
            }
        }

        impl<'a, I: Ix> From<BaseModel<&'a $graph<I>, from_impls!(@ref $players, I, 'a)>>
            for BaseModelEnum<
                BaseModel<&'a ChoiceGraph<I>, SinglePlayer>,
                BaseModel<&'a BranchGraph<I>, SinglePlayer>,
                BaseModel<&'a ChoiceBranchGraph<I>, SinglePlayer>,
                BaseModel<&'a ChoiceGraph<I>, &'a TurnBased<I, TwoPlayer>>,
                BaseModel<&'a ChoiceBranchGraph<I>, &'a TurnBased<I, TwoPlayer>>,
            >
        {
            fn from(value: BaseModel<&'a $graph<I>, from_impls!(@ref $players, I, 'a)>) -> Self {
                Self::$variant(value)
            }
        }

        impl<'a, I: Ix> From<BaseModel<&'a mut $graph<I>, from_impls!(@mut $players, I, 'a)>>
            for BaseModelEnum<
                BaseModel<&'a mut ChoiceGraph<I>, SinglePlayer>,
                BaseModel<&'a mut BranchGraph<I>, SinglePlayer>,
                BaseModel<&'a mut ChoiceBranchGraph<I>, SinglePlayer>,
                BaseModel<&'a mut ChoiceGraph<I>, &'a mut TurnBased<I, TwoPlayer>>,
                BaseModel<&'a mut ChoiceBranchGraph<I>, &'a mut TurnBased<I, TwoPlayer>>,
            >
        {
            fn from(
                value: BaseModel<&'a mut $graph<I>, from_impls!(@mut $players, I, 'a)>,
            ) -> Self {
                Self::$variant(value)
            }
        }
    };
    // `SinglePlayer` is returned by value from `as_ref` and `as_mut`
    (@owned SinglePlayer, $I:ident) => { SinglePlayer };
    (@owned TwoPlayer, $I:ident) => { TurnBased<$I, TwoPlayer> };
    (@ref SinglePlayer, $I:ident, $a:lifetime) => { SinglePlayer };
    (@ref TwoPlayer, $I:ident, $a:lifetime) => { &$a TurnBased<$I, TwoPlayer> };
    (@mut SinglePlayer, $I:ident, $a:lifetime) => { SinglePlayer };
    (@mut TwoPlayer, $I:ident, $a:lifetime) => { &$a mut TurnBased<$I, TwoPlayer> };
}

from_impls!(TransitionSystem, ChoiceGraph, SinglePlayer);
from_impls!(Dtmc, BranchGraph, SinglePlayer);
from_impls!(Mdp, ChoiceBranchGraph, SinglePlayer);
from_impls!(NonstochasticGame, ChoiceGraph, TwoPlayer);
from_impls!(StochasticGame, ChoiceBranchGraph, TwoPlayer);
