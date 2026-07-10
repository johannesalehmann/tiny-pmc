use crate::ExpressionContext;
use crate::expressions::VariableType;
use crate::variables::const_valuations::ConstValuations;
use crate::variables::valuation_map::{ValuationMap, ValuationMapEntry};
use prism_model::{Span, VariableManager};
use probabilistic_models::ValuationClassEntryIndex;
use typed_index_collections::{Index, RawIndex, To1};

pub struct VariableDetail {
    pub bounds: Option<(i64, i64)>,
    pub variable_type: VariableType,
}

pub struct VariableDetails<ClassEntryIdx: Index> {
    pub details: To1<ClassEntryIdx, VariableDetail>,
}
impl<ClassEntryIdx: Index> VariableDetails<ClassEntryIdx> {
    #[cfg(test)]
    pub fn with_mock_values() -> Self {
        Self {
            details: To1::with_entries(vec![
                VariableDetail {
                    bounds: None,
                    variable_type: VariableType::Float,
                },
                VariableDetail {
                    bounds: Some((-10, 15)),
                    variable_type: VariableType::Int,
                },
                VariableDetail {
                    bounds: None,
                    variable_type: VariableType::Bool,
                },
            ]),
        }
    }

    pub fn new<S: Span, E, EC: ExpressionContext<E>>(
        variables: &VariableManager<S, E>,
        valuation_map: &ValuationMap<()>,
        const_values: &ConstValuations,
        expression_context: &mut EC,
    ) -> Self {
        let mut details = To1::new();
        let const_value_source = super::ConstOnlyValuationSource::new(valuation_map, const_values);

        for (i, variable) in variables.variables.iter().enumerate() {
            if let ValuationMapEntry::Var(index) = valuation_map[i] {
                let bounds = match &variable.range {
                    prism_model::VariableRange::BoundedInt { min, max, .. } => {
                        let min = expression_context.evaluate_int(min, &const_value_source);
                        let max = expression_context.evaluate_int(max, &const_value_source);
                        Some((min, max))
                    }
                    _ => None,
                };

                let variable_type = VariableType::from_range(&variable.range);

                // TODO: Everything assumes that this add_unchecked returns the same index as the
                //  similar call that adds the variable to the ValuationClass. In practice, this is
                //  the case, but it should ideally be checked at runtime (or just ensured
                //  statically, if there is a way to do this).
                details.add(VariableDetail {
                    bounds,
                    variable_type,
                });
            }
        }

        Self { details }
    }
}

impl<ClassEntryIdx: Index> std::ops::Index<ClassEntryIdx> for VariableDetails<ClassEntryIdx> {
    type Output = VariableDetail;

    fn index(&self, index: ClassEntryIdx) -> &Self::Output {
        &self.details[index]
    }
}
