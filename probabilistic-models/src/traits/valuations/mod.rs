use crate::valuations::{ModelValuationEntry, ValuationClass, ValuationLookupTable, Valuations};
use crate::{HasIx, Ix, Model, StateIndex, ValuationClassEntryIndex, ValuationClassIndex};
use typed_index_collections::SemiboundedIndexRange;

pub trait ReadValuations: HasIx {
    fn state_valuation(&self, state: StateIndex<Self::Ix>) -> ModelValuationEntry<'_, Self::Ix>;

    fn state_valuations(
        &self,
    ) -> impl Iterator<Item = (StateIndex<Self::Ix>, ModelValuationEntry<'_, Self::Ix>)>;

    fn generate_valuation_lookup(&self) -> ValuationLookupTable<Self::Ix, StateIndex<Self::Ix>>;

    fn valuation_classes(&self) -> SemiboundedIndexRange<ValuationClassIndex<Self::Ix>>;

    fn valuation_class(
        &self,
        class: ValuationClassIndex<Self::Ix>,
    ) -> &ValuationClass<ValuationClassEntryIndex<Self::Ix>>;
}

macro_rules! derive_read_valuations {
    ($subcomponent:ident) => {
        fn state_valuation(
            &self,
            state: $crate::StateIndex<Self::Ix>,
        ) -> $crate::valuations::ModelValuationEntry<'_, Self::Ix> {
            self.$subcomponent.state_valuation(state)
        }

        fn state_valuations(
            &self,
        ) -> impl Iterator<
            Item = (
                $crate::StateIndex<Self::Ix>,
                $crate::valuations::ModelValuationEntry<'_, Self::Ix>,
            ),
        > {
            self.$subcomponent.state_valuations()
        }

        fn generate_valuation_lookup(
            &self,
        ) -> ValuationLookupTable<Self::Ix, StateIndex<Self::Ix>> {
            self.$subcomponent.generate_valuation_lookup()
        }

        fn valuation_classes(
            &self,
        ) -> $crate::typed_index_collections::SemiboundedIndexRange<
            $crate::ValuationClassIndex<Self::Ix>,
        > {
            self.$subcomponent.valuation_classes()
        }

        fn valuation_class(
            &self,
            class: $crate::ValuationClassIndex<Self::Ix>,
        ) -> &$crate::valuations::ValuationClass<$crate::ValuationClassEntryIndex<Self::Ix>> {
            self.$subcomponent.valuation_class(class)
        }
    };
}
pub(crate) use derive_read_valuations;

impl<I: Ix> ReadValuations for Valuations<I> {
    fn state_valuation(&self, state: StateIndex<I>) -> ModelValuationEntry<'_, I> {
        self.entry(state)
    }

    fn state_valuations(
        &self,
    ) -> impl Iterator<Item = (StateIndex<I>, ModelValuationEntry<'_, I>)> {
        self.entries()
    }

    fn generate_valuation_lookup(&self) -> ValuationLookupTable<Self::Ix, StateIndex<Self::Ix>> {
        ValuationLookupTable::from_valuations(self)
    }

    fn valuation_classes(&self) -> SemiboundedIndexRange<ValuationClassIndex<I>> {
        self.classes()
    }

    fn valuation_class(
        &self,
        class: ValuationClassIndex<I>,
    ) -> &ValuationClass<ValuationClassEntryIndex<I>> {
        self.class(class)
    }
}

impl<
    M: HasIx,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    StateVals: ReadValuations<Ix = M::Ix>,
    Preds,
> ReadValuations for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    derive_read_valuations!(state_valuations);
}

impl<T: ReadValuations> ReadValuations for &T {
    fn state_valuation(&self, state: StateIndex<Self::Ix>) -> ModelValuationEntry<'_, Self::Ix> {
        (**self).state_valuation(state)
    }

    fn state_valuations(
        &self,
    ) -> impl Iterator<Item = (StateIndex<Self::Ix>, ModelValuationEntry<'_, Self::Ix>)> {
        (**self).state_valuations()
    }

    fn generate_valuation_lookup(&self) -> ValuationLookupTable<Self::Ix, StateIndex<Self::Ix>> {
        (**self).generate_valuation_lookup()
    }

    fn valuation_classes(&self) -> SemiboundedIndexRange<ValuationClassIndex<Self::Ix>> {
        (**self).valuation_classes()
    }

    fn valuation_class(
        &self,
        class: ValuationClassIndex<Self::Ix>,
    ) -> &ValuationClass<ValuationClassEntryIndex<Self::Ix>> {
        (**self).valuation_class(class)
    }
}

impl<T: ReadValuations> ReadValuations for &mut T {
    fn state_valuation(&self, state: StateIndex<Self::Ix>) -> ModelValuationEntry<'_, Self::Ix> {
        (**self).state_valuation(state)
    }

    fn state_valuations(
        &self,
    ) -> impl Iterator<Item = (StateIndex<Self::Ix>, ModelValuationEntry<'_, Self::Ix>)> {
        (**self).state_valuations()
    }

    fn generate_valuation_lookup(&self) -> ValuationLookupTable<Self::Ix, StateIndex<Self::Ix>> {
        (**self).generate_valuation_lookup()
    }

    fn valuation_classes(&self) -> SemiboundedIndexRange<ValuationClassIndex<Self::Ix>> {
        (**self).valuation_classes()
    }

    fn valuation_class(
        &self,
        class: ValuationClassIndex<Self::Ix>,
    ) -> &ValuationClass<ValuationClassEntryIndex<Self::Ix>> {
        (**self).valuation_class(class)
    }
}

pub trait ReadValuationsMaybe: HasIx {
    type WithValuations: ReadValuations<Ix = Self::Ix>;

    fn has_valuations(&self) -> bool;
    fn try_with_valuations(self) -> Option<Self::WithValuations>;
}

impl<T: ReadValuations> ReadValuationsMaybe for T {
    type WithValuations = T;

    fn has_valuations(&self) -> bool {
        true
    }

    fn try_with_valuations(self) -> Option<Self::WithValuations> {
        Some(self)
    }
}

impl<T: ReadValuations> ReadValuationsMaybe for Option<T> {
    type WithValuations = T;

    fn has_valuations(&self) -> bool {
        self.is_some()
    }

    fn try_with_valuations(self) -> Option<Self::WithValuations> {
        self
    }
}

impl<
    M: HasIx,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    StateVals: ReadValuations<Ix = M::Ix>,
    Preds,
> ReadValuationsMaybe
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Option<StateVals>, Preds>
{
    type WithValuations = Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>;

    fn has_valuations(&self) -> bool {
        self.state_valuations.is_some()
    }

    fn try_with_valuations(self) -> Option<Self::WithValuations> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations?,
            predecessors: self.predecessors,
        })
    }
}
