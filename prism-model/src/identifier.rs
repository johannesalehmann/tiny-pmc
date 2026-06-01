use crate::module::RenameRules;
use crate::spans::Span;
use crate::{Displayable, FullSpan};
use std::fmt::{Display, Formatter};

/// An identifier, i.e. a name with associated [`Span`].
///
/// # Example
///
/// Construct an identifier by calling [`Identifier::new()`]. This checks whether the identifier is
/// valid and returns a [`InvalidName`] error if
/// - [the identifier is empty](InvalidName::Empty),
/// - [the identifier starts with a number](InvalidName::StartsWithNumber),
/// - [the identifier contains an invalid character](InvalidName::InvalidCharacter) or
/// - [the identifier is a reserved keyword of the PRISM language](InvalidName::Reserved)
///     (a list of reserved keywords is available in the [PRISM documentation](https://www.prismmodelchecker.org/manual/ThePRISMLanguage/ModulesAndVariables)).
/// ```
/// # use prism_model::{FullSpan, Identifier, InvalidName, Span};
/// let var_name = Identifier::new("x");
/// assert_eq!(var_name, Ok(Identifier { name: "x".to_string(), span: FullSpan::empty()}));
///
/// let empty: Result<Identifier, InvalidName> = Identifier::new("");
/// assert_eq!(empty, Err(InvalidName::Empty));
///
/// let starts_with_number: Result<Identifier, InvalidName> = Identifier::new("3rd_value");
/// assert_eq!(starts_with_number, Err(InvalidName::StartsWithNumber {character: '3'}));
///
/// let invalid_character: Result<Identifier, InvalidName> = Identifier::new("München");
/// assert_eq!(invalid_character, Err(InvalidName::InvalidCharacter {location: 1, character: 'ü'}));
///
/// let reserved: Result<Identifier, InvalidName> = Identifier::new("A");
/// assert_eq!(reserved, Err(InvalidName::Reserved {name: "A".to_string()}));
/// ```
///
/// To avoid the check for reserved keywords, use [`Identifier::new_potentially_reserved`]. This is
/// particularly useful when constructing functions calls with
///
/// ```
/// # use prism_model::{Expression, Identifier, InvalidName};
/// let min_function = Identifier::new_potentially_reserved("min").expect("Invalid identifier");
/// let min: Expression
///     = Expression::function(min_function, vec![Expression::int(3), Expression::int(5)]);
/// ```
#[derive(Clone)]
pub struct Identifier<S: Span = FullSpan> {
    /// The name of the identifier, for example the name of the variable or module identified by
    /// this identifier.
    pub name: String,

    /// The [`Span`] of the identifier.
    pub span: S,
}

impl<S: Span> std::fmt::Debug for Identifier<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<S: Span> Display for Identifier<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<S: Span> crate::private::Sealed for Identifier<S> {}
impl<S: Span> Displayable<()> for Identifier<S> {
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &()) -> std::fmt::Result {
        let _ = context;
        self.fmt(f)
    }
}

impl<S: Span> Identifier<S> {
    const RESERVED_NAMES: &'static [&'static str] = &[
        "A",
        "bool",
        "clock",
        "const",
        "ctmc",
        "C",
        "double",
        "dtmc",
        "E",
        "endinit",
        "endinvariant",
        "endmodule",
        "endobservables",
        "endrewards",
        "endsystem",
        "false",
        "formula",
        "filter",
        "func",
        "F",
        "global",
        "G",
        "init",
        "invariant",
        "I",
        "int",
        "label",
        "max",
        "mdp",
        "min",
        "module",
        "X",
        "nondeterministic",
        "observable",
        "observables",
        "of",
        "Pmax",
        "Pmin",
        "P",
        "pomdp",
        "popta",
        "probabilistic",
        "prob",
        "pta",
        "rate",
        "rewards",
        "Rmax",
        "Rmin",
        "R",
        "S",
        "stochastic",
        "system",
        "true",
        "U",
        "W",
    ];

    /// Constructs an identifier with given name and empty [`Span`].
    ///
    /// This checks whether the name is legal and returns [`InvalidName`] if it is not. See
    /// [`Identifier`] for usage examples.
    ///
    /// To construct an identifier with reserved name (e.g. to identify a function), use
    /// [`Identifier::new_potentially_reserved()`].
    ///
    /// To construct an identifier with custom span, use [`Identifier::new_spanned()`].
    pub fn new<Str: Into<String>>(name: Str) -> Result<Self, InvalidName> {
        Self::new_with_reserved_option(name, false, S::empty())
    }

    /// Constructs an identifier with given name and [`Span`].
    ///
    /// This checks whether the name is legal and returns [`InvalidName`] if it is not. See
    /// [`Identifier`] for usage examples.
    ///
    /// To construct an identifier with reserved name (e.g. to identify a function), use
    /// [`Identifier::new_potentially_reserved_spanned()`].
    ///
    /// To construct an identifier with empty span, use [`Identifier::new()`].
    pub fn new_spanned<Str: Into<String>>(
        name: Str,
        span: S,
    ) -> Result<Self, crate::identifier::InvalidName> {
        Self::new_with_reserved_option(name, false, span)
    }

    /// Constructs an identifier with given name and empty [`Span`]. Here, `name` may be a reserved
    /// keyword.
    ///
    /// Identifiers for reserved keywords are mainly useful for function calls, as PRISM functions
    /// have reserved keywords as names (e.g. `min`, `pow`, `floor`).
    ///
    /// The remaining checks for name validity are still performed and [`InvalidName`] is returned
    /// on failure. See [`Identifier`] for usage examples.
    ///
    /// To construct an identifier that must not contain a reserved keyword, use
    /// [`Identifier::new()`].
    ///
    /// To construct an identifier with custom span, use
    /// [`Identifier::new_potentially_reserved_spanned()`].
    pub fn new_potentially_reserved<Str: Into<String>>(
        name: Str,
    ) -> Result<Self, crate::identifier::InvalidName> {
        Self::new_with_reserved_option(name, true, S::empty())
    }

    /// Constructs an identifier with the given name and [`Span`]. Here `name` may be a reserved
    /// keyword.
    ///
    /// Apart from including a custom span, behaviour is identical to
    /// [`Identifier::new_potentially_reserved()`]. Refer to that function's documentation for
    /// details.
    pub fn new_potentially_reserved_spanned<Str: Into<String>>(
        name: Str,
        span: S,
    ) -> Result<Self, crate::identifier::InvalidName> {
        Self::new_with_reserved_option(name, true, span)
    }

    fn new_with_reserved_option<Str: Into<String>>(
        name: Str,
        allow_reserved: bool,
        span: S,
    ) -> Result<Self, InvalidName> {
        let name: String = name.into();
        let first_character = name.chars().nth(0);
        match first_character {
            Some(character) => {
                if character.is_ascii_digit() {
                    Err(InvalidName::StartsWithNumber { character })
                } else {
                    Ok(())
                }
            }
            None => Err(InvalidName::Empty),
        }?;

        for (location, character) in name.chars().enumerate() {
            if !(character.is_ascii_alphanumeric() || character == '_') {
                return Err(InvalidName::InvalidCharacter {
                    location,
                    character,
                });
            }
        }

        if !allow_reserved {
            if Self::RESERVED_NAMES.contains(&name.as_str()) {
                return Err(InvalidName::Reserved { name: name.clone() });
            }
        }

        Ok(Self { name, span })
    }

    /// Maps the [`Span`] of this `Identifier` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// Usage is analogous to [`Expression::map_span()`](crate::Expression::map_span). Refer to its
    /// documentation for details and examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Identifier<S2> {
        Identifier {
            name: self.name,
            span: map(self.span),
        }
    }

    /// Constructs a new identifier by applying `rename_rules` to self.
    ///
    /// # Example
    ///
    /// Let `rename_rules` contain the rules `x` -> `y` and `alpha` -> `beta`.
    /// ```
    /// # use prism_model::{RenameRules, RenameRule, Identifier};
    /// let rename_rules: RenameRules = RenameRules::with_rules(vec![
    ///     RenameRule::new(Identifier::new("x").unwrap(), Identifier::new("y").unwrap()),
    ///     RenameRule::new(Identifier::new("alpha").unwrap(), Identifier::new("beta").unwrap()),
    /// ]);
    /// ```
    ///
    /// Then applying this to identifiers `x` and `alpha` changes them accordingly, whereas `beta`
    /// is left unaffected.
    ///
    /// ```
    /// # use prism_model::{RenameRules, RenameRule, Identifier};
    /// let rename_rules: RenameRules = RenameRules::with_rules(vec![
    /// #     RenameRule::new(Identifier::new("x").unwrap(), Identifier::new("y").unwrap()),
    /// #     RenameRule::new(Identifier::new("alpha").unwrap(), Identifier::new("beta").unwrap()),
    /// # ]);
    /// let x = Identifier::new("x").unwrap();
    /// assert_eq!(x.renamed(&rename_rules), Identifier::new("y").unwrap());
    ///
    /// let alpha = Identifier::new("alpha").unwrap();
    /// assert_eq!(alpha.renamed(&rename_rules), Identifier::new("beta").unwrap());
    ///
    /// let beta = Identifier::new("beta").unwrap();
    /// assert_eq!(beta.renamed(&rename_rules), Identifier::new("beta").unwrap());
    /// ```
    ///
    /// # Usage
    ///
    /// Instead of calling this individually on every identifier in a model, use the appropriate
    /// helper method, e.g. [`Command::renamed()`](crate::Command::renamed()).
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        rename_rules.get_renaming(self).unwrap_or(self.clone())
    }
}

impl<S: Span> PartialEq for Identifier<S> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// Error indicating that a string is not a
/// [valid PRISM identifier](https://www.prismmodelchecker.org/manual/ThePRISMLanguage/ModulesAndVariables).
///
/// See [`Identifier`] for examples.
#[derive(Debug, PartialEq, Clone)]
pub enum InvalidName {
    /// The identifier is empty
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::{Identifier, InvalidName};
    /// let empty: Result<Identifier, InvalidName> = Identifier::new("");
    /// assert_eq!(empty, Err(InvalidName::Empty));
    /// ```
    Empty,

    /// The identifier starts with a number
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::{Identifier, InvalidName};
    /// let starts_with_number: Result<Identifier, InvalidName> = Identifier::new("3rd_value");
    /// assert_eq!(starts_with_number, Err(InvalidName::StartsWithNumber {character: '3'}));
    /// ```
    StartsWithNumber {
        /// The digit that the identifier starts with
        character: char,
    },

    /// The identifier contains an invalid character. Legal characters are `A`-`Z`, `a`-`z`, `0`-`9`
    /// and `_`.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::{Identifier, InvalidName};
    /// let invalid_character: Result<Identifier, InvalidName> = Identifier::new("München");
    /// assert_eq!(invalid_character, Err(InvalidName::InvalidCharacter {location: 1, character: 'ü'}));
    ///
    /// let invalid_hyphen: Result<Identifier, InvalidName> = Identifier::new("main-module");
    /// assert_eq!(invalid_hyphen, Err(InvalidName::InvalidCharacter {location: 4, character: '-'}));
    /// ```
    InvalidCharacter {
        /// The location of the first invalid character.
        location: usize,
        /// The invalid character
        character: char,
    },

    /// The identifier is a [reserved keyword](https://www.prismmodelchecker.org/manual/ThePRISMLanguage/ModulesAndVariables).
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_model::{Identifier, InvalidName};
    /// let reserved: Result<Identifier, InvalidName> = Identifier::new("A");
    /// assert_eq!(reserved, Err(InvalidName::Reserved {name: "A".to_string()}));
    ///
    /// let reserved: Result<Identifier, InvalidName> = Identifier::new("min");
    /// assert_eq!(reserved, Err(InvalidName::Reserved {name: "min".to_string()}));
    /// ```
    ///
    /// To construct an identifier with reserved name, e.g. in order to identify a function,
    /// use [`Identifier::new_potentially_reserved()`]:
    ///
    /// ```
    /// # use prism_model::{FullSpan, Identifier, InvalidName, Span};
    /// let min_function = Identifier::new_potentially_reserved("min");
    /// assert_eq!(min_function, Ok(Identifier{ name: "min".to_string(), span: FullSpan::empty() }));
    /// ```
    Reserved {
        /// The reserved keyword
        name: String,
    },
}
