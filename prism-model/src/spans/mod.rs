use std::ops::Range;

pub trait Span: Clone {
    fn from_start_end(start: usize, end: usize) -> Self {
        Self::from_range(start..end)
    }
    fn from_range(range: Range<usize>) -> Self;
    fn empty() -> Self;
    fn join(span_1: &Self, span_2: &Self) -> Self;

    fn range(&self) -> Option<Range<usize>>;

    fn start(&self) -> Option<usize> {
        self.range().map(|r| r.start)
    }
    fn end(&self) -> Option<usize> {
        self.range().map(|r| r.end)
    }
}

impl Span for () {
    fn from_range(range: Range<usize>) -> Self {
        let _ = range;
        ()
    }

    fn empty() -> Self {
        ()
    }

    fn join(span_1: &Self, span_2: &Self) -> Self {
        let _ = (span_1, span_2);
        ()
    }

    fn range(&self) -> Option<Range<usize>> {
        None
    }
}

#[derive(Copy, Clone)]
pub struct FullSpan {
    inner: FullSpanInner,
}

impl FullSpan {
    // TODO: This function only exists to reduce migration burden. In the future, error formatting
    //  should gracefully handle models without spans by not adding syntax highlighting for the
    //  relevant parts of the model.
    pub fn into_range(self) -> Range<usize> {
        match self.inner {
            FullSpanInner::Empty => {
                panic!("Span is empty")
            }
            FullSpanInner::Full { start, end } => start..end,
        }
    }
}

#[derive(Copy, Clone)]
enum FullSpanInner {
    Empty,
    Full { start: usize, end: usize },
}

impl Span for FullSpan {
    fn from_range(range: Range<usize>) -> Self {
        Self {
            inner: FullSpanInner::Full {
                start: range.start,
                end: range.end,
            },
        }
    }

    fn empty() -> Self {
        Self {
            inner: FullSpanInner::Empty,
        }
    }

    fn join(span_1: &Self, span_2: &Self) -> Self {
        match (&span_1.inner, &span_2.inner) {
            (
                FullSpanInner::Full {
                    start: start_1,
                    end: end_1,
                },
                FullSpanInner::Full {
                    start: start_2,
                    end: end_2,
                },
            ) => Self {
                inner: FullSpanInner::Full {
                    start: (*start_1).min(*start_2),
                    end: (*end_1).max(*end_2),
                },
            },
            _ => Self {
                inner: FullSpanInner::Empty,
            },
        }
    }

    fn range(&self) -> Option<Range<usize>> {
        match self.inner {
            FullSpanInner::Empty => None,
            FullSpanInner::Full { start, end } => Some(start..end),
        }
    }
}

// TODO: Hide this behind a feature flag
impl chumsky::span::Span for FullSpan {
    type Context = ();
    type Offset = usize;

    fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
        let _ = context;
        Self::from_range(range)
    }

    fn context(&self) -> Self::Context {
        ()
    }

    fn start(&self) -> Self::Offset {
        match self.inner {
            FullSpanInner::Empty => 0,
            FullSpanInner::Full { start, .. } => start,
        }
    }

    fn end(&self) -> Self::Offset {
        match self.inner {
            FullSpanInner::Empty => 0,
            FullSpanInner::Full { end, .. } => end,
        }
    }
}
