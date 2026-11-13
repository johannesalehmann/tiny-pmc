use crate::labels::{ChoiceLabels, ReadLabels};
use crate::{ActionIndex, ChoiceIndex, HasIx, Ix, Model};

pub trait ReadChoiceLabels: HasIx {
    type E;

    fn choice_label(&self, choice: ChoiceIndex<Self::Ix>) -> &Self::E;
    fn choice_action_index(&self, choice: ChoiceIndex<Self::Ix>) -> ActionIndex<Self::Ix>;
    fn label_of_choice_action(&self, action: ActionIndex<Self::Ix>) -> &Self::E;
}

macro_rules! derive_read_choice_labels {
    ($subcomponent:ident) => {
        fn choice_label(&self, choice: $crate::ChoiceIndex<Self::Ix>) -> &Self::E {
            self.$subcomponent.choice_label(choice)
        }

        fn choice_action_index(
            &self,
            choice: $crate::ChoiceIndex<Self::Ix>,
        ) -> $crate::ActionIndex<Self::Ix> {
            self.$subcomponent.choice_action_index(choice)
        }

        fn label_of_choice_action(&self, action: $crate::ActionIndex<Self::Ix>) -> &Self::E {
            self.$subcomponent.label_of_choice_action(action)
        }
    };
}
pub(crate) use derive_read_choice_labels;

impl<I: Ix, E> ReadChoiceLabels for ChoiceLabels<I, E> {
    type E = E;

    fn choice_label(&self, choice: ChoiceIndex<Self::Ix>) -> &Self::E {
        self.label(choice)
    }

    fn choice_action_index(&self, choice: ChoiceIndex<Self::Ix>) -> ActionIndex<Self::Ix> {
        self.action_index(choice)
    }

    fn label_of_choice_action(&self, action: ActionIndex<Self::Ix>) -> &Self::E {
        self.label_of_action(action)
    }
}

impl<
    M: HasIx,
    Ini,
    ChLabel: ReadChoiceLabels<Ix = M::Ix>,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    StateVals,
    Preds,
> ReadChoiceLabels for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type E = ChLabel::E;

    derive_read_choice_labels!(choice_labels);
}

impl<T: ReadChoiceLabels> ReadChoiceLabels for &T {
    type E = T::E;

    fn choice_label(&self, choice: ChoiceIndex<Self::Ix>) -> &Self::E {
        (**self).choice_label(choice)
    }
    fn choice_action_index(&self, choice: ChoiceIndex<Self::Ix>) -> ActionIndex<Self::Ix> {
        (**self).choice_action_index(choice)
    }
    fn label_of_choice_action(&self, action: ActionIndex<Self::Ix>) -> &Self::E {
        (**self).label_of_choice_action(action)
    }
}

impl<T: ReadChoiceLabels> ReadChoiceLabels for &mut T {
    type E = T::E;

    fn choice_label(&self, choice: ChoiceIndex<Self::Ix>) -> &Self::E {
        (**self).choice_label(choice)
    }
    fn choice_action_index(&self, choice: ChoiceIndex<Self::Ix>) -> ActionIndex<Self::Ix> {
        (**self).choice_action_index(choice)
    }
    fn label_of_choice_action(&self, action: ActionIndex<Self::Ix>) -> &Self::E {
        (**self).label_of_choice_action(action)
    }
}

pub trait ReadChoiceLabelsMaybe: HasIx {
    type E;
    type WithChoiceLabels: ReadChoiceLabels<Ix = Self::Ix, E = Self::E>;

    fn has_choice_labels(&self) -> bool;
    fn try_with_choice_labels(self) -> Option<Self::WithChoiceLabels>;
}

impl<T: ReadChoiceLabels> ReadChoiceLabelsMaybe for T {
    type E = T::E;
    type WithChoiceLabels = T;

    fn has_choice_labels(&self) -> bool {
        true
    }

    fn try_with_choice_labels(self) -> Option<Self::WithChoiceLabels> {
        Some(self)
    }
}

impl<T: ReadChoiceLabels> ReadChoiceLabelsMaybe for Option<T> {
    type E = T::E;
    type WithChoiceLabels = T;

    fn has_choice_labels(&self) -> bool {
        self.is_some()
    }

    fn try_with_choice_labels(self) -> Option<Self::WithChoiceLabels> {
        self
    }
}

impl<
    M: HasIx,
    Ini,
    ChLabel: ReadChoiceLabels<Ix = M::Ix>,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    StateVals,
    Preds,
> ReadChoiceLabelsMaybe
    for Model<M, Ini, Option<ChLabel>, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type E = ChLabel::E;
    type WithChoiceLabels = Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>;

    fn has_choice_labels(&self) -> bool {
        self.choice_labels.is_some()
    }

    fn try_with_choice_labels(self) -> Option<Self::WithChoiceLabels> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels?,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        })
    }
}
