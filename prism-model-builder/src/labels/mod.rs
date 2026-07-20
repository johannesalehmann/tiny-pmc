use crate::expressions::stack_based_expressions::{StackBasedExpression, SubExpressionManager};
use prism_model::{Expression, Identifier, Label, Model, Span, VariableManager, VariableReference};
use std::collections::HashSet;
use typed_index_collections::{Index, NamedTo1};

pub struct Labels<APIdx: Index, E> {
    labels: NamedTo1<APIdx, E>,
    sealed: bool,
}

impl<APIdx: Index, E> Default for Labels<APIdx, E> {
    fn default() -> Self {
        Self {
            labels: NamedTo1::default(),
            sealed: false,
        }
    }
}

impl<APIdx: Index, E> Labels<APIdx, E> {
    pub fn get_or_add(&mut self, name: String, value: E) -> APIdx {
        if let Some(index) = self.labels.index_by_name(&name) {
            index
        } else {
            if self.sealed {
                panic!(
                    "The property requires labels (`{name}`) that are not present in the selection of labels to be built."
                )
            }
            self.labels.add_entry(name, value)
        }
    }

    pub fn next_free_name(&mut self, prefix: String) -> String {
        let mut index = None;
        loop {
            let name = if let Some(index) = index {
                format!("{prefix}_{index}")
            } else {
                prefix.clone()
            };
            if !self.labels.contains_name(&name) {
                return name;
            }
            index = if let Some(index) = index {
                Some(index + 1)
            } else {
                Some(0)
            };
        }
    }

    fn seal(&mut self) {
        self.sealed = true;
    }

    pub fn map_e<E2>(self, map: impl FnMut(E) -> E2) -> Labels<APIdx, E2> {
        Labels {
            labels: self.labels.map(map),
            sealed: false,
        }
    }
}
impl<APIdx: Index, S: Span> Labels<APIdx, Expression<VariableReference, S>> {
    pub fn to_stack_based<E>(
        self,
        sub_expression_manager: &mut SubExpressionManager<VariableReference>,
        variable_manager: &VariableManager<S, E>,
    ) -> Labels<APIdx, usize> {
        self.map_e(|condition| {
            let stack_expression =
                StackBasedExpression::from_expression(&condition, variable_manager);
            sub_expression_manager.add_sub_expression(stack_expression)
        })
    }
}

impl<'a, APIdx: Index, E> IntoIterator for &'a Labels<APIdx, E> {
    type Item = (&'a str, &'a E);
    type IntoIter = typed_index_collections::NamedTo1Iterator<'a, APIdx, E>;

    fn into_iter(self) -> Self::IntoIter {
        self.labels.into_iter()
    }
}

pub trait LabelSource {
    fn extract_labels<S: Span, APIdx: Index>(
        &self,
        model: &Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
    ) -> Labels<APIdx, Expression<VariableReference, S>>;
}

#[derive(Default)]
pub struct AllLabels {}

impl LabelSource for AllLabels {
    fn extract_labels<S: Span, APIdx: Index>(
        &self,
        model: &Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
    ) -> Labels<APIdx, Expression<VariableReference, S>> {
        let mut labels = Labels::default();
        for label in &model.labels {
            labels.get_or_add(label.name.name.clone(), label.condition.clone());
        }
        labels
    }
}

#[derive(Default)]
pub struct NoLabels {}

impl LabelSource for NoLabels {
    fn extract_labels<S: Span, APIdx: Index>(
        &self,
        _model: &Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
    ) -> Labels<APIdx, Expression<VariableReference, S>> {
        let mut labels = Labels::default();
        labels.seal();
        labels
    }
}

#[derive(Default)]
pub struct OnlyNecessary {}

impl LabelSource for OnlyNecessary {
    fn extract_labels<S: Span, APIdx: Index>(
        &self,
        _model: &Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
    ) -> Labels<APIdx, Expression<VariableReference, S>> {
        let labels = Labels::default();
        labels
    }
}

pub struct ListedLabels {
    names: HashSet<String>,
}

impl LabelSource for ListedLabels {
    fn extract_labels<S: Span, APIdx: Index>(
        &self,
        model: &Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
    ) -> Labels<APIdx, Expression<VariableReference, S>> {
        let mut labels = ListedPlusNecessaryLabels {
            names: self.names.clone(),
        }
        .extract_labels(model);
        labels.seal();
        labels
    }
}

pub struct ListedPlusNecessaryLabels {
    names: HashSet<String>,
}

impl LabelSource for ListedPlusNecessaryLabels {
    fn extract_labels<S: Span, APIdx: Index>(
        &self,
        model: &Model<VariableReference, S, Expression<VariableReference, S>, Identifier<S>>,
    ) -> Labels<APIdx, Expression<VariableReference, S>> {
        let mut labels = Labels::default();
        for label in &model.labels {
            if self.names.contains(&label.name.name) {
                labels.get_or_add(label.name.name.clone(), label.condition.clone());
            }
        }
        labels
    }
}
