use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use typed_index_collections::RawIndex;

/// An index profile stores the raw integer types that back the model's various index types.
pub trait Ix: Copy + Clone + Debug + Default + PartialEq + Eq + Hash + PartialOrd + Ord {
    /// Width of state-sized indices.
    type State: RawIndex;
    /// Width of choice-sized indices.
    type Choice: RawIndex;
    /// Width of branch-sized indices.
    type Branch: RawIndex;
    /// Width of remaining indices. In particular, this includes name-addressed collections such as
    /// atomic propositions.
    type Meta: RawIndex;
}

/// Profile that uses `R` for state, choice and branch indices, and `M` for meta indices.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Uniform<R, M = u32>(PhantomData<(R, M)>);

impl<R: RawIndex, M: RawIndex> Ix for Uniform<R, M> {
    type State = R;
    type Choice = R;
    type Branch = R;
    type Meta = M;
}

pub type DefaultIx = Uniform<usize>;

/// Profile with the same widths as `I`, but distinct index types. Use this for a model derived
/// from another one (e.g. a sub-model), so that indices of the two cannot be mixed up.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sub<I>(PhantomData<I>);

impl<I: Ix> Ix for Sub<I> {
    type State = I::State;
    type Choice = I::Choice;
    type Branch = I::Branch;
    type Meta = I::Meta;
}

/// Implemented by models and model components to declare their index profile. All `Read...`
/// traits have this as a supertrait, so all components read through a single type (e.g. a
/// `Model`) necessarily share the same profile.
pub trait HasIx {
    type Ix: Ix;
}

impl<T: HasIx + ?Sized> HasIx for &T {
    type Ix = T::Ix;
}

impl<T: HasIx + ?Sized> HasIx for &mut T {
    type Ix = T::Ix;
}

impl<T: HasIx> HasIx for Option<T> {
    type Ix = T::Ix;
}
