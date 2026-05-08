use crate::spans::Span;
use crate::{Expression, Identifier, IdentityMapExpression};

pub struct LabelSubstitutionVisitor<'a, S: Span> {
    pub label_name: &'a Identifier<S>,
    pub expression: &'a Expression<Identifier<S>, S>,
}

impl<'a, S: Span> crate::expressions::identity_map::Private for LabelSubstitutionVisitor<'a, S> {}
impl<'a, S: Span> IdentityMapExpression<crate::Identifier<S>, S>
    for LabelSubstitutionVisitor<'a, S>
{
    fn visit_label(&mut self, name: Identifier<S>, span: S) -> Expression<Identifier<S>, S> {
        if &name == self.label_name {
            self.expression.clone()
        } else {
            Expression::Label(name, span)
        }
    }
}
