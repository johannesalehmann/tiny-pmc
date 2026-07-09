mod distributions;

pub use distributions::Distribution;
use std::collections::HashMap;

use crate::Index;
use crate::annotations::distributions::IdentityDistribution;
use crate::index::RawIndex;
use crate::named_collection::NamedTo1;
use crate::to1::To1;
use crate::{AnnotationEntryIndex, AnnotationIndex, StateIndex};
use std::marker::PhantomData;

#[derive(Default)]
pub struct TypedAnnotation<
    I: RawIndex,
    From: Index,
    Dist: Distribution<From, AnnotationEntryIndex<I>>,
    Val,
> {
    pub distribution: Dist,
    pub values: To1<AnnotationEntryIndex<I>, Val>,
    pub phantom_data: PhantomData<From>,
}

impl<I: RawIndex, From: Index, Val>
    TypedAnnotation<I, From, IdentityDistribution<From, AnnotationEntryIndex<I>>, Val>
{
    pub fn add_value(&mut self, entity: From, value: Val) {
        let annotation_index = self.distribution.annotation_index(entity);
        self.values.add(annotation_index, value);
    }
}

pub enum Annotation<I: RawIndex, From: Index, Dist: Distribution<From, AnnotationEntryIndex<I>>> {
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

pub type AnnotationGroup<
    I: RawIndex,
    From: Index,
    Dist: Distribution<From, AnnotationEntryIndex<I>>,
> = NamedTo1<AnnotationIndex<I>, Annotation<I, From, Dist>>;

pub type TypedAnnotationGroup<
    I: RawIndex,
    From: Index,
    Dist: Distribution<From, AnnotationEntryIndex<I>>,
    Val,
> = NamedTo1<AnnotationIndex<I>, TypedAnnotation<I, From, Dist, Val>>;

pub type AtomicPropositions<I: RawIndex> = TypedAnnotationGroup<
    I,
    StateIndex<I>,
    IdentityDistribution<StateIndex<I>, AnnotationEntryIndex<I>>,
    bool,
>;

pub struct Rewards<States, Choices, Branches> {
    pub states: States,
    pub choices: Choices,
    pub branches: Branches,
}
