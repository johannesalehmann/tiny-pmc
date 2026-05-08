use std::fmt::{Display, Formatter};

pub trait Displayable<Ctx>: private::Sealed {
    fn displayable<'a, 'b>(
        &'a self,
        context: &'b Ctx,
    ) -> DisplayableWithContext<'a, 'b, Self, Ctx> {
        DisplayableWithContext {
            element: self,
            context,
        }
    }

    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result;
}

pub struct DisplayableWithContext<'a, 'b, O: ?Sized + Displayable<Ctx>, Ctx> {
    element: &'a O,
    context: &'b Ctx,
}

pub mod private {
    pub trait Sealed {}
}

impl<O: ?Sized + Displayable<Ctx>, Ctx> Display for DisplayableWithContext<'_, '_, O, Ctx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.element.fmt_internal(f, self.context)
    }
}
