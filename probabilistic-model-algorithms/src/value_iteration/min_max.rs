use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{ReadOwners, ReadStateSpace};
use std::marker::PhantomData;

pub trait ValueComparator {
    type Model: ReadStateSpace;

    fn initial_value(
        &self,
        state: <Self::Model as ReadStateSpace>::StateIdx,
        model: &Self::Model,
    ) -> f64;
    fn is_better(
        &self,
        state: <Self::Model as ReadStateSpace>::StateIdx,
        model: &Self::Model,
        before: f64,
        new: f64,
    ) -> bool;
}

pub struct Maximiser<M> {
    _phantom_data: PhantomData<M>,
}

impl<M: ReadStateSpace> Default for Maximiser<M> {
    fn default() -> Self {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

impl<M: ReadStateSpace> ValueComparator for Maximiser<M> {
    type Model = M;

    fn initial_value(
        &self,
        _state: <Self::Model as ReadStateSpace>::StateIdx,
        _model: &Self::Model,
    ) -> f64 {
        0.0
    }

    fn is_better(
        &self,
        _state: <Self::Model as ReadStateSpace>::StateIdx,
        _model: &Self::Model,
        before: f64,
        new: f64,
    ) -> bool {
        new >= before
    }
}

pub struct Minimiser<M> {
    _phantom_data: PhantomData<M>,
}

impl<M: ReadStateSpace> Default for Minimiser<M> {
    fn default() -> Self {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

impl<M: ReadStateSpace> ValueComparator for Minimiser<M> {
    type Model = M;

    fn initial_value(
        &self,
        _state: <Self::Model as ReadStateSpace>::StateIdx,
        _model: &Self::Model,
    ) -> f64 {
        1.0
    }

    fn is_better(
        &self,
        _state: <Self::Model as ReadStateSpace>::StateIdx,
        _model: &Self::Model,
        before: f64,
        new: f64,
    ) -> bool {
        new <= before
    }
}

pub struct PlayerOneMaximisesPlayerTwoMinimises<M> {
    _phantom_data: PhantomData<M>,
}

impl<M> Default for PlayerOneMaximisesPlayerTwoMinimises<M> {
    fn default() -> Self {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

impl<
    M: ReadStateSpace + ReadOwners<OwnerType = TwoPlayer, StateIdx = <M as ReadStateSpace>::StateIdx>,
> ValueComparator for PlayerOneMaximisesPlayerTwoMinimises<M>
{
    type Model = M;

    fn initial_value(
        &self,
        state: <Self::Model as ReadStateSpace>::StateIdx,
        model: &Self::Model,
    ) -> f64 {
        match model.state_owner(state) {
            TwoPlayer::Eve => 0.0,
            TwoPlayer::Adam => 1.0,
        }
    }

    fn is_better(
        &self,
        state: <Self::Model as ReadStateSpace>::StateIdx,
        model: &Self::Model,
        before: f64,
        new: f64,
    ) -> bool {
        match model.state_owner(state) {
            TwoPlayer::Eve => new >= before,
            TwoPlayer::Adam => new <= before,
        }
    }
}
