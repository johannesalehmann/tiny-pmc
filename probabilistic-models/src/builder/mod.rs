// use crate::index::RawIndex;
// use crate::{ChoiceIndex, Model, StateIndex};
//
// pub struct ModelBuilderBuilder<I: RawIndex, Base: BaseModelBuilder<I>> {
//     base: Base,
//     choice_labeler: ChoiceLabeler,
//     branch_labeler: BranchLabeler,
//     atomic_propositions: APBuilder,
// }
//
// pub struct ModelBuilder<I: RawIndex, Base: BaseModelBuilder<I>, Ini: InitialStatesBuilder<I>> {
//     base: Base,
//     initial_states: Ini,
// }
//
// impl ModelBuilder<I: RawIndex, Base: BaseModelBuilder<I>> {}
//
// pub trait BaseModelBuilder<I: RawIndex> {
//     type BaseModel;
//
//     fn finish_choice(choice_index: ChoiceIndex<I>);
//     fn finish_state(state_index: StateIndex<I>);
//     fn add_choice(); // TODO: Figure out parameters
// }
//
// pub trait InitialStatesBuilder<I: RawIndex> {
//     type InitialStates;
//
//     fn stores_initial_states() -> bool;
//     fn mark_state(&mut self, state: StateIndex<I>);
// }
//
// pub trait ChoiceLabeler<I: RawIndex> {
//     type ChoiceLabels;
//
//     fn stores_choice_labels() -> bool;
//     fn label_choice(&mut self, choice: ChoiceIndex<I>, label: Option<String>);
// }
//
// pub trait AtomicPropositionBuilder<I: RawIndex> {
//     type AtomicPropositions;
//
//     fn stores_atomic_propositions() -> bool;
//     fn set_value(&mut self, id: String, state: StateIndex<I>, value: bool);
// }
