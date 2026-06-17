pub use ariadne::Source;
use ariadne::{Label, Report, ReportBuilder, ReportKind};
use prism_model::Span;
use std::ops::Range;

pub struct MaybeReportBuilder<S: Span> {
    kind: MaybeReportKind,
    message: Option<String>,
    notes: Vec<String>,
    help: Vec<String>,
    location: S,
    labels: Vec<MaybeLabel<S>>,
}

impl<S: Span> MaybeReportBuilder<S> {
    pub fn new_error(location: &S) -> Self {
        Self::new(MaybeReportKind::Error, location)
    }

    #[allow(unused)]
    pub fn new_warning(location: &S) -> Self {
        Self::new(MaybeReportKind::Warning, location)
    }

    #[allow(unused)]
    pub fn new_advice(location: &S) -> Self {
        Self::new(MaybeReportKind::Advice, location)
    }

    pub fn new(kind: MaybeReportKind, location: &S) -> Self {
        Self {
            kind,
            message: None,
            notes: Vec::new(),
            help: Vec::new(),
            location: location.clone(),
            labels: Vec::new(),
        }
    }

    pub fn set_message<Str: Into<String>>(&mut self, message: Str) {
        self.message = Some(message.into());
    }

    pub fn add_note<Str: Into<String>>(&mut self, note: Str) {
        self.notes.push(note.into());
    }

    pub fn add_help<Str: Into<String>>(&mut self, help: Str) {
        self.help.push(help.into());
    }

    pub fn add_label(&mut self, label: MaybeLabel<S>) {
        self.labels.push(label);
    }

    pub fn to_ariadne_builder<'a>(
        mut self,
        file_name: &'a str,
    ) -> ReportBuilder<'a, (&'a str, Range<usize>)> {
        let span = self.location.range().unwrap_or_else(|| {
            self.notes.push(
                "No span was stored for this error. A span links a model to its source code"
                    .to_string(),
            );
            0..1
        });
        let mut builder = Report::build(self.kind.to_ariadne_kind(), (file_name, span));
        if let Some(message) = self.message {
            builder.set_message(message);
        }
        for label in self.labels {
            match label.span.range() {
                Some(range) => {
                    let mut ariadne_label = Label::new((file_name, range));
                    if let Some(msg) = label.message {
                        ariadne_label = ariadne_label.with_message(msg);
                    };
                    builder.add_label(ariadne_label);
                }
                None => {
                    if let Some(msg) = label.message {
                        self.notes.push(
                            format!("{msg} (this message refers to a component without associated source code span)"));
                    }
                }
            }
        }
        for note in self.notes {
            builder.add_note(note);
        }
        for help in self.help {
            builder.add_help(help);
        }

        builder
    }
}

pub enum MaybeReportKind {
    Error,
    Warning,
    Advice,
}

impl MaybeReportKind {
    pub fn to_ariadne_kind<'a>(&self) -> ReportKind<'a> {
        match self {
            MaybeReportKind::Error => ReportKind::Error,
            MaybeReportKind::Warning => ReportKind::Warning,
            MaybeReportKind::Advice => ReportKind::Advice,
        }
    }
}

pub struct MaybeLabel<S: Span> {
    span: S,
    message: Option<String>,
}

impl<S: Span> MaybeLabel<S> {
    pub fn new(location: &S) -> Self {
        Self {
            span: location.clone(),
            message: None,
        }
    }

    pub fn with_message<Str: Into<String>>(mut self, message: Str) -> Self {
        self.message = Some(message.into());
        self
    }
}
