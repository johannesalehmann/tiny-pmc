mod distributions;

pub use distributions::Distribution;

use crate::annotations::distributions::IdentityDistribution;
use std::marker::PhantomData;
use typed_index_collections::{Index, MappedIndices, NamedTo1, To1, To1BoolValues};

#[derive(Default)]
pub struct TypedAnnotation<
    EntityIdx: Index,
    AnnotationEntryIdx: Index,
    Dist: Distribution<EntityIdx, AnnotationEntryIdx>,
    Val,
> {
    distribution: Dist,
    values: To1<AnnotationEntryIdx, Val>,
    phantom_data: PhantomData<EntityIdx>,
}

impl<EntityIdx: Index, AnnotationEntryIdx: Index, Val>
    TypedAnnotation<
        EntityIdx,
        AnnotationEntryIdx,
        IdentityDistribution<EntityIdx, AnnotationEntryIdx>,
        Val,
    >
{
    pub fn add_value(&mut self, entity: EntityIdx, value: Val) {
        let annotation_index = self.distribution.annotation_index(entity);
        self.values.add_checked(annotation_index, value);
    }

    pub fn get(&self, entity: EntityIdx) -> Option<&Val> {
        let annotation_index = self.distribution.annotation_index(entity);
        self.values.get(annotation_index)
    }

    pub fn get_mut(&mut self, entity: EntityIdx) -> Option<&mut Val> {
        let annotation_index = self.distribution.annotation_index(entity);
        self.values.get_mut(annotation_index)
    }
}
impl<EntityIdx: Index, AnnotationEntryIdx: Index>
    TypedAnnotation<
        EntityIdx,
        AnnotationEntryIdx,
        IdentityDistribution<EntityIdx, AnnotationEntryIdx>,
        bool,
    >
{
    pub fn true_values(
        &self,
    ) -> To1BoolValues<EntityIdx, MappedIndices<'_, AnnotationEntryIdx, EntityIdx, bool>> {
        self.values.with_key_type().true_values()
    }
}

impl<EntityIdx: Index, AnnotationEntryIdx: Index, Val> std::ops::Index<EntityIdx>
    for TypedAnnotation<
        EntityIdx,
        AnnotationEntryIdx,
        IdentityDistribution<EntityIdx, AnnotationEntryIdx>,
        Val,
    >
{
    type Output = Val;

    fn index(&self, index: EntityIdx) -> &Self::Output {
        let annotation_index = self.distribution.annotation_index(index);
        &self.values[annotation_index]
    }
}

impl<EntityIdx: Index, AnnotationEntryIdx: Index, Val> std::ops::IndexMut<EntityIdx>
    for TypedAnnotation<
        EntityIdx,
        AnnotationEntryIdx,
        IdentityDistribution<EntityIdx, AnnotationEntryIdx>,
        Val,
    >
{
    fn index_mut(&mut self, index: EntityIdx) -> &mut Self::Output {
        let annotation_index = self.distribution.annotation_index(index);
        &mut self.values[annotation_index]
    }
}

pub enum Annotation<
    From: Index,
    AnnotationEntryIdx: Index,
    Dist: Distribution<From, AnnotationEntryIdx>,
> {
    Boolean(TypedAnnotation<From, AnnotationEntryIdx, Dist, bool>),
    Int(TypedAnnotation<From, AnnotationEntryIdx, Dist, i64>),
    UInt(TypedAnnotation<From, AnnotationEntryIdx, Dist, u64>),
    IntInterval(TypedAnnotation<From, AnnotationEntryIdx, Dist, (i64, i64)>),
    UIntInterval(TypedAnnotation<From, AnnotationEntryIdx, Dist, (u64, u64)>),
    Double(TypedAnnotation<From, AnnotationEntryIdx, Dist, f64>),
    DoubleInterval(TypedAnnotation<From, AnnotationEntryIdx, Dist, (f64, f64)>),
    Rational(TypedAnnotation<From, AnnotationEntryIdx, Dist, (i64, u64)>),
    RationalInterval(TypedAnnotation<From, AnnotationEntryIdx, Dist, ((i64, u64), (i64, u64))>),
    String(TypedAnnotation<From, AnnotationEntryIdx, Dist, String>),
}

pub type AnnotationGroup<AnnotationIdx, EntityIdx, AnnotationEntryIdx, Dist> =
    NamedTo1<AnnotationIdx, Annotation<EntityIdx, AnnotationEntryIdx, Dist>>;

pub type TypedAnnotationGroup<AnnotationIdx, EntityIdx, AnnotationEntryIdx, Dist, Val> =
    NamedTo1<AnnotationIdx, TypedAnnotation<EntityIdx, AnnotationEntryIdx, Dist, Val>>;

pub type AtomicPropositions<AnnotationIdx, StateIdx, AnnotationEntryIdx> = TypedAnnotationGroup<
    AnnotationIdx,
    StateIdx,
    AnnotationEntryIdx,
    IdentityDistribution<StateIdx, AnnotationEntryIdx>,
    bool,
>;

pub struct Rewards<States, Choices, Branches> {
    pub states: States,
    pub choices: Choices,
    pub branches: Branches,
}
