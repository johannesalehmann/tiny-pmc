use std::marker::PhantomData;
use typed_index_collections::{Csr, Index, IndexRange, To1};

/// Maps entities (e.g. states) to the entries of an annotation that hold their values.
pub trait Distribution<From: Index> {
    /// Index of the entries of an annotation using this distribution.
    type Entry: Index;
    fn annotations_of_state(&self, state: From) -> IndexRange<Self::Entry>;
    fn probability(&self, index: Self::Entry) -> f32;
}

#[allow(unused)] // TODO: Remove once probabilistic annotations are supported
#[derive(Default)]
pub struct ProbabilisticDistribution<From: Index, To: Index> {
    entity_to_annotations: Csr<From, To>,
    probabilities: To1<To, f32>,
}

impl<From: Index, To: Index> Distribution<From> for ProbabilisticDistribution<From, To> {
    type Entry = To;

    fn annotations_of_state(&self, state: From) -> IndexRange<To> {
        self.entity_to_annotations.index(state)
    }

    fn probability(&self, index: To) -> f32 {
        self.probabilities[index]
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct IdentityDistribution<From: Index, To: Index = From> {
    phantom_data: PhantomData<(From, To)>,
}

impl<From: Index, To: Index> IdentityDistribution<From, To> {
    pub fn annotation_index(&self, from: From) -> To {
        from.reinterpret_as()
    }
}

impl<From: Index, To: Index> Distribution<From> for IdentityDistribution<From, To> {
    type Entry = To;

    fn annotations_of_state(&self, state: From) -> IndexRange<To> {
        IndexRange::with_single_index(state.reinterpret_as())
    }

    fn probability(&self, _index: To) -> f32 {
        1.0
    }
}

#[allow(unused)] // TODO: Remove once probabilistic annotations are supported
pub enum MixedDistribution<From: Index, To: Index> {
    Probabilistic(ProbabilisticDistribution<From, To>),
    Identity(IdentityDistribution<From, To>),
}

impl<From: Index, To: Index> Distribution<From> for MixedDistribution<From, To> {
    type Entry = To;

    fn annotations_of_state(&self, state: From) -> IndexRange<To> {
        match self {
            MixedDistribution::Probabilistic(dist) => dist.annotations_of_state(state),
            MixedDistribution::Identity(dist) => dist.annotations_of_state(state),
        }
    }

    fn probability(&self, index: To) -> f32 {
        match self {
            MixedDistribution::Probabilistic(dist) => dist.probability(index),
            MixedDistribution::Identity(dist) => dist.probability(index),
        }
    }
}
