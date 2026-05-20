use std::ops::Range;

/// Trait representing a contiguous range (a "span") of source code.
///
/// A span marks the section of PRISM code that corresponds to an element in the model. Most
/// components of [`crate::Model`] have an associated span. This is mainly useful for printing error
/// messages and for annotating the source code with rich information.
///
/// # Creating spans
///
/// Spans can be created with [`Span::from_start_end()`] and [`Span::from_range()`]. Two spans can be
/// combined into one span encompassing both with [`Span::join()`].
///
/// For some components, it does not make sense to construct a span, e.g. when parsing a model from
/// disk (with spans) and then adding additional modules programmatically (which do not correspond
/// to any source code). Therefore, `Span` provides [`Span::empty()`] to construct an empty span.
///
/// # Accessing spans
///
/// Spans are accessed with  [`Span::range()`], [`Span::start()`] and [`Span::end()`].
///
/// These function return `Option`s. The cases in which `None` is returned depend on the
/// implementation (see below).
///
/// # Implementations
///
/// [`FullSpan`] represents a span that is either empty or has a start and end location. For most
/// purposes, this is a sensible default. It only returns `None` from the access functions when
/// constructed with [`Span::empty`].
///
/// Span is also implemented for `()`. This is useful for models without a span, e.g. because they
/// were generated entirely programmatically. As `()` stores no information, all access methods
/// return `None`, even when constructed with e.g. [`Span::from_range()`].
///
/// Future implementations may return `Some` even when constructed with [`Span::empty()`]. For
/// example, it may instantiate `start` and `end` with default values and return these instead o
/// `None`.
pub trait Span: Clone {
    /// Constructs a span given a start and end index. The start index is inclusive, the end index
    /// is exclusive.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::Span;
    /// fn spans<S: Span>() {
    ///     let src = "mdp\nmodule main\nendmodule";
    ///     let mdp_span = S::from_start_end(0, 3);
    ///     let main_span = S::from_start_end(11, 15);
    /// }
    /// ```
    fn from_start_end(start: usize, end: usize) -> Self {
        Self::from_range(start..end)
    }

    /// Constructs a span given a range.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::Span;
    /// fn spans<S: Span>() {
    ///     let src = "mdp\nmodule main\nendmodule";
    ///     let mdp_span = S::from_range(0..3);
    ///     let main_span = S::from_range(11..15);
    /// }
    /// ```
    fn from_range(range: Range<usize>) -> Self;

    /// Constructs an empty span.
    ///
    /// Depending on the implementation, this may be stored explicitly or may be translated into
    /// a default span, e.g. `0..0`.
    ///
    /// # Example
    ///
    /// The following function takes an expression `expr` (which may include spans) and constructs
    /// `expr + 1`. As neither `1` nor `expr + 1` correspond to any location in the source, we use
    /// `S::empty` as the span.
    ///
    /// ```
    /// # use prism_model::*;
    /// fn plus_one<S: Span>(expr: ExpressionNamedVars<S>) -> ExpressionNamedVars<S>{
    ///     let one = Expression::int_spanned(1, S::empty());
    ///     expr.plus_spanned(one, S::empty())
    /// }
    /// ```
    ///
    /// The following function is equivalent, as e.g. [`Expression::int(val)`] calls
    /// `Expression::int_spanned(val, S::empty)`:
    ///
    /// ```
    /// # use prism_model::*;
    /// fn plus_one_simplified<S: Span>(expr: ExpressionNamedVars<S>) -> ExpressionNamedVars<S>{
    ///     let one = Expression::int(1);
    ///     expr.plus(one)
    /// }
    /// ```
    fn empty() -> Self;

    /// Joins two spans into one span encompassing both.
    ///
    /// As spans only represent contiguous sections, the resulting span may cover code that neither
    /// `span_1` nor `span_2` covers.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::*;
    /// fn plus<V, S: Span>(lhs: Expression<V, S>, rhs: Expression<V, S>) -> Expression<V, S> {
    ///     let plus_span = S::join(lhs.span(), rhs.span());
    ///     lhs.plus_spanned(rhs, plus_span)
    /// }
    ///
    /// let x: Expression = Expression::int_spanned(123, FullSpan::from_range(32..35));
    /// let y: Expression = Expression::int_spanned(456, FullSpan::from_range(36..39));
    /// let x_plus_y = plus(x, y);
    /// assert_eq!(x_plus_y.span().range(), Some(32..39));
    /// ```
    fn join(span_1: &Self, span_2: &Self) -> Self;

    /// Returns the range associated with the span, if it exists and is tracked.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::{FullSpan, Span};
    /// let full = FullSpan::from_start_end(5, 20);
    /// assert_eq!(full.range(), Some(5..20));
    ///
    /// let full_empty = FullSpan::empty();
    /// assert_eq!(full_empty.range(), None);
    /// ```
    ///
    /// Using `()` as a span always returns `None`.
    ///
    /// ```
    /// # use prism_model::{Span};
    /// let untracked = <()>::from_range(7..12);
    /// assert_eq!(untracked.range(), None);
    /// ```
    fn range(&self) -> Option<Range<usize>>;

    /// Returns the (inclusive) start index of the span.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::{FullSpan, Span};
    /// let full = FullSpan::from_range(2..4);
    /// assert_eq!(full.start(), Some(2));
    ///
    /// let full_empty = FullSpan::empty();
    /// assert_eq!(full_empty.start(), None);
    /// ```
    ///
    /// Using `()` as a span always returns `None`.
    ///
    /// ```
    /// # use prism_model::{Span};
    /// let untracked = <()>::from_range(7..12);
    /// assert_eq!(untracked.start(), None);
    /// ```
    fn start(&self) -> Option<usize> {
        self.range().map(|r| r.start)
    }

    /// Returns the (exclusive) end index of the span.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::{FullSpan, Span};
    /// let full = FullSpan::from_range(2..4);
    /// assert_eq!(full.end(), Some(4));
    ///
    /// let full_empty = FullSpan::empty();
    /// assert_eq!(full_empty.end(), None);
    /// ```
    ///
    /// Using `()` as a span always returns `None`.
    ///
    /// ```
    /// # use prism_model::Span;
    /// let untracked = <()>::from_range(7..12);
    /// assert_eq!(untracked.end(), None);
    /// ```
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

// TODO:  Implement `Debug` in a nicer, custom way
/// Either represents a contiguous section in the source code or the empty span.
///
/// See [`Span`] for general notes on creating and accessing spans.
///
/// # Example
///
/// ```
/// # use prism_model::{FullSpan, Span};
/// let span_1 = FullSpan::from_start_end(5, 8);
/// let span_2 = FullSpan::from_range(3..4);
/// let joined = FullSpan::join(&span_1, &span_2);
///
/// assert_eq!(joined.range(), Some(3..8));
/// assert_eq!(joined.start(), Some(3));
/// assert_eq!(joined.end(), Some(8));
///
/// let empty = FullSpan::empty();
/// assert_eq!(empty.range(), None);
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FullSpan {
    inner: FullSpanInner,
}

impl FullSpan {
    // TODO: This function only exists to reduce migration burden. In the future, error formatting
    //  should gracefully handle models without spans by not adding syntax highlighting for the
    //  relevant parts of the model.
    /// Returns the range stored by the `FullSpan`.
    ///
    /// This function should not be used. Instead, call [`Span::range()`]. It only serves to reduce
    /// the migration burden of the existing PRISM parser. Once this is updated, the function will
    /// be removed.
    ///
    /// # Panics
    ///
    /// If the `FullSpan` represents the empty span, this function panics. Use [`Span::range()`] to
    /// gracefully handle this case.
    #[deprecated(note = "Use `Span::range()` instead.")]
    pub fn into_range(self) -> Range<usize> {
        match self.inner {
            FullSpanInner::Empty => {
                panic!("Span is empty")
            }
            FullSpanInner::Full { start, end } => start..end,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
