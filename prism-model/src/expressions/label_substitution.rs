use crate::spans::Span;
use crate::{Expression, Identifier, IdentityMapExpression};

pub struct LabelSubstitutionVisitor<'a, V, S: Span> {
    pub label_name: &'a Identifier<S>,
    pub expression: &'a Expression<V, S>,
}

impl<'a, V, S: Span> crate::expressions::identity_map::Private
    for LabelSubstitutionVisitor<'a, V, S>
{
}
impl<'a, V: Clone, S: Span> IdentityMapExpression<V, S> for LabelSubstitutionVisitor<'a, V, S> {
    fn visit_label(&mut self, name: Identifier<S>, span: S) -> Expression<V, S> {
        if &name == self.label_name {
            self.expression.clone()
        } else {
            Expression::Label(name, span)
        }
    }
}
