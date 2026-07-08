mod distributions;

pub use distributions::Distribution;
use std::collections::HashMap;

use crate::Index;
use crate::annotations::distributions::IdentityDistribution;
use crate::index::RawIndex;
use crate::to1::To1;
use crate::{AnnotationIndex, StateIndex};
use std::marker::PhantomData;

pub struct TypedAnnotation<
    I: RawIndex,
    From: Index,
    Dist: Distribution<From, AnnotationIndex<I>>,
    Val,
> {
    distribution: Dist,
    values: To1<AnnotationIndex<I>, Val>,
    phantom_data: PhantomData<From>,
}

pub enum Annotation<I: RawIndex, From: Index, Dist: Distribution<From, AnnotationIndex<I>>> {
    Boolean(TypedAnnotation<I, From, Dist, bool>),
    Int(TypedAnnotation<I, From, Dist, i64>),
    UInt(TypedAnnotation<I, From, Dist, u64>),
    IntInterval(TypedAnnotation<I, From, Dist, (i64, i64)>),
    UIntInterval(TypedAnnotation<I, From, Dist, (u64, u64)>),
    Double(TypedAnnotation<I, From, Dist, f64>),
    DoubleInterval(TypedAnnotation<I, From, Dist, (f64, f64)>),
    Rational(TypedAnnotation<I, From, Dist, (i64, u64)>),
    RationalInterval(TypedAnnotation<I, From, Dist, ((i64, u64), (i64, u64))>),
    String(TypedAnnotation<I, From, Dist, String>),
}

pub struct AnnotationGroup<I: RawIndex, From: Index, Dist: Distribution<From, AnnotationIndex<I>>> {
    entries: HashMap<String, Annotation<I, From, Dist>>,
}

pub struct TypedAnnotationGroup<
    I: RawIndex,
    From: Index,
    Dist: Distribution<From, AnnotationIndex<I>>,
    Val,
> {
    entries: HashMap<String, TypedAnnotation<I, From, Dist, Val>>,
}

pub type AtomicPropositions<I: RawIndex> = TypedAnnotationGroup<
    I,
    StateIndex<I>,
    IdentityDistribution<StateIndex<I>, AnnotationIndex<I>>,
    bool,
>;

pub struct Rewards<States, Choices, Branches> {
    states: States,
    choices: Choices,
    branches: Branches,
}
