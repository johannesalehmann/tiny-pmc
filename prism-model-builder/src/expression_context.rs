use crate::expressions::stack_based_expressions::SubExpressionProvider;
use crate::expressions::{TreeWalkingEvaluator, ValuationSource};
use prism_model::{Expression, Span, VariableReference};

pub trait ExpressionContext<E> {
    fn reset_context(&mut self);

    fn evaluate_int<V: ValuationSource>(&mut self, expression: &E, valuations: &V) -> i64;
    fn evaluate_int_with_separate_context<V: ValuationSource>(
        &self,
        expression: &E,
        valuations: &V,
    ) -> i64;
    fn evaluate_float<V: ValuationSource>(&mut self, expression: &E, valuations: &V) -> f64;
    fn evaluate_float_with_separate_context<V: ValuationSource>(
        &self,
        expression: &E,
        valuations: &V,
    ) -> f64;
    fn evaluate_bool<V: ValuationSource>(&mut self, expression: &E, valuations: &V) -> bool;
    fn evaluate_bool_with_separate_context<V: ValuationSource>(
        &self,
        expression: &E,
        valuations: &V,
    ) -> bool;
}

impl<S: Span> ExpressionContext<Expression<VariableReference, S>> for TreeWalkingEvaluator {
    fn reset_context(&mut self) {}

    fn evaluate_int<V: ValuationSource>(
        &mut self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> i64 {
        self.evaluate_as_int(expression, valuations)
    }

    fn evaluate_int_with_separate_context<V: ValuationSource>(
        &self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> i64 {
        self.evaluate_as_int(expression, valuations)
    }

    fn evaluate_float<V: ValuationSource>(
        &mut self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> f64 {
        self.evaluate_as_float(expression, valuations)
    }

    fn evaluate_float_with_separate_context<V: ValuationSource>(
        &self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> f64 {
        self.evaluate_as_float(expression, valuations)
    }

    fn evaluate_bool<V: ValuationSource>(
        &mut self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> bool {
        self.evaluate_as_bool(expression, valuations)
    }

    fn evaluate_bool_with_separate_context<V: ValuationSource>(
        &self,
        expression: &Expression<VariableReference, S>,
        valuations: &V,
    ) -> bool {
        self.evaluate_as_bool(expression, valuations)
    }
}

pub struct SubExpressionExpressionContext<'a, SE: SubExpressionProvider> {
    pub sub_expressions: &'a SE,
    pub context: SE::EvaluationContext,
}

impl<'a, SE: SubExpressionProvider> ExpressionContext<usize>
    for SubExpressionExpressionContext<'a, SE>
{
    fn reset_context(&mut self) {
        self.sub_expressions.reset_context(&mut self.context);
    }

    fn evaluate_int<V: ValuationSource>(&mut self, expression: &usize, valuations: &V) -> i64 {
        self.sub_expressions
            .evaluate_as_int(*expression, valuations, &mut self.context)
    }

    fn evaluate_int_with_separate_context<V: ValuationSource>(
        &self,
        expression: &usize,
        valuations: &V,
    ) -> i64 {
        let mut context = self.sub_expressions.create_context();
        self.sub_expressions
            .evaluate_as_int(*expression, valuations, &mut context)
    }

    fn evaluate_float<V: ValuationSource>(&mut self, expression: &usize, valuations: &V) -> f64 {
        self.sub_expressions
            .evaluate_as_float(*expression, valuations, &mut self.context)
    }

    fn evaluate_float_with_separate_context<V: ValuationSource>(
        &self,
        expression: &usize,
        valuations: &V,
    ) -> f64 {
        let mut context = self.sub_expressions.create_context();
        self.sub_expressions
            .evaluate_as_float(*expression, valuations, &mut context)
    }

    fn evaluate_bool<V: ValuationSource>(&mut self, expression: &usize, valuations: &V) -> bool {
        self.sub_expressions
            .evaluate_as_bool(*expression, valuations, &mut self.context)
    }

    fn evaluate_bool_with_separate_context<V: ValuationSource>(
        &self,
        expression: &usize,
        valuations: &V,
    ) -> bool {
        let mut context = self.sub_expressions.create_context();
        self.sub_expressions
            .evaluate_as_bool(*expression, valuations, &mut context)
    }
}
