use crate::{FullSpan, Identifier, Span};
use std::collections::HashMap;

/// Collection of attributes. Each attribute either has form `key` or `key=value`.
///
/// Attributes are not a feature of the PRISM language. They are present here to support use-cases
/// that need additional information (e.g. program repair, where you want to mark which components
/// should be repaired).
#[derive(PartialEq, Clone, Debug)]
pub struct Attributes<S: Span = FullSpan> {
    attributes: HashMap<String, Attribute<S>>,
}

impl<S: Span> Attributes<S> {
    /// Constructs and empty set of attributes.
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    /// Constructs a set of attributes with the given content.
    pub fn with_attributes(attributes: Vec<Attribute<S>>) -> Result<Self, AttributeExists<S>> {
        let mut res = Self::new();
        for attribute in attributes {
            res.add(attribute)?;
        }
        Ok(res)
    }

    /// Adds an attribute to the collection.
    ///
    /// If an attribute with the same key exists, [`AttributeExists`] is returned.
    pub fn add(&mut self, attribute: Attribute<S>) -> Result<(), AttributeExists<S>> {
        if let Some(existing) = self.attributes.get(&attribute.key.name) {
            Err(AttributeExists {
                existing_name: existing.key.clone(),
                new_name: attribute.key.clone(),
                existing_span: existing.span.clone(),
                new_span: attribute.span,
            })
        } else {
            self.attributes
                .insert(attribute.key.name.clone(), attribute);
            Ok(())
        }
    }

    /// Returns the attribute with the given key if it exists, otherwise `None`.
    pub fn get(&self, key: &str) -> Option<&Attribute<S>> {
        self.attributes.get(key)
    }

    /// Returns the value of the attribute with the given key if the attribute exists and has a
    /// value (i.e. the attribute is not a flag). Otherwise, returns `None`.
    pub fn value(&self, key: &str) -> Option<&AttributeValue<S>> {
        self.get(key).and_then(|a| a.value.as_ref())
    }

    /// Returns a mutable reference to the value of the attribute with the given key if the
    /// attribute exists and has a value (i.e. the attribute is not a flag). Otherwise, returns
    /// `None`.
    pub fn value_mut(&mut self, key: &str) -> Option<&mut AttributeValue<S>> {
        self.attributes.get_mut(key).and_then(|a| a.value.as_mut())
    }

    /// Returns `true` if an attribute with the given key exists and is a flag (i.e. the attribute
    /// has no value). If the attribute does not exist or has a value, returns `false`.
    pub fn is_flag_set(&self, key: &str) -> bool {
        if let Some(attribute) = self.get(key) {
            if attribute.value.is_none() {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Returns an iterator over the attributes.
    pub fn iter(&self) -> impl Iterator<Item = &Attribute<S>> {
        (&self).into_iter()
    }

    /// Maps every [`Span`] of this `Attributes` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to every [`Attribute`] stored in this collection.
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Attributes<S2> {
        let mut res = Attributes::new();
        for (key, attribute) in self.attributes {
            res.attributes.insert(key, attribute.map_span(map));
        }
        res
    }
}

impl<'a, S: Span> IntoIterator for &'a Attributes<S> {
    type Item = &'a Attribute<S>;
    type IntoIter = std::collections::hash_map::Values<'a, String, Attribute<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.values()
    }
}

impl<S: Span> IntoIterator for Attributes<S> {
    type Item = Attribute<S>;
    type IntoIter = std::collections::hash_map::IntoValues<String, Attribute<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.into_values()
    }
}

/// An attribute with the same name already exists
#[derive(Clone, PartialEq, Debug)]
pub struct AttributeExists<S: Span> {
    /// The identifier of the existing attribute.
    ///
    /// The [`name`](Identifier::name) is identical to that of [`new_name`](Self::new_name), but the
    /// [`Span`] covers the definition of the existing name.
    pub existing_name: Identifier<S>,

    /// The identifier of the new attribute.
    ///
    /// The [`name`](Identifier::name) is identical to that of
    /// [`existing_name`](Self::existing_name), but the [`Span`] covers the definition of the new
    /// name.
    pub new_name: Identifier<S>,

    /// The [`Span`] of the entire existing definition.
    ///
    /// To access just the span of the existing key, use
    /// [`existing_name`](Self::existing_name)`.`[`span`](Identifier::span)
    pub existing_span: S,

    /// The [`Span`] of the entire new definition.
    ///
    /// To access just the span of the existing key, use
    /// [`existing_name`](Self::existing_name)`.`[`span`](Identifier::span)
    pub new_span: S,
}

/// An attribute, consisting of a key, an optional value and a [`Span`] that covers the attribute
/// definition.
#[derive(PartialEq, Clone, Debug)]
pub struct Attribute<S: Span = FullSpan> {
    /// The key of the attribute
    pub key: Identifier<S>,

    /// [`Span`] covering the entire attribute definition. To get just the span of the key or value,
    /// refer to their respective `.span` fields.
    pub span: S,

    /// The value of the attribute. If this is `None`, then the attribute is a flag, otherwise, it
    /// is a key-value-pair.
    pub value: Option<AttributeValue<S>>,
}

impl<S: Span> Attribute<S> {
    /// Constructs a flag attribute, i.e. one without a value, with an empty [`Span`].
    ///
    /// To provide a custom span, use [`Self::flag_spanned()`]. To construct an attribute with a
    /// value, use [`Self::key_value()`].
    pub fn flag(key: Identifier<S>) -> Self {
        Self::flag_spanned(key, S::empty())
    }

    /// Constructs a flag attribute, i.e. one without a value, with the given [`Span`].
    ///
    /// To use an empty span, use [`Self::flag()`]. To construct an attribute with a value, use
    /// [`Self::key_value_spanned()`].
    pub fn flag_spanned(key: Identifier<S>, span: S) -> Self {
        Self {
            key,
            span,
            value: None,
        }
    }

    /// Constructs an attribute with given key and value and with empty [`Span`].
    ///
    /// To provide a custom span, use [`Self::key_value_spanned()`]. To construct a flag attribute
    /// (i.e. an attribute without a value), use [`Self::flag()`].
    pub fn key_value<Str: Into<String>>(key: Identifier<S>, value: Str) -> Self {
        Self::key_value_spanned(key, value, S::empty(), S::empty())
    }

    /// Constructs an attribute with given key and value and [`Span`].
    ///
    /// To use an empty span, use [`Self::key_value()`]. To construct a flag attribute (i.e. an
    /// attribute without a value), use [`Self::flag_spanned()`].
    pub fn key_value_spanned<Str: Into<String>>(
        key: Identifier<S>,
        value: Str,
        span: S,
        value_span: S,
    ) -> Self {
        Self {
            key,
            span,
            value: Some(AttributeValue {
                value: value.into(),
                span: value_span,
            }),
        }
    }

    /// Maps every [`Span`] of this `Attribute` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is to [`key`](Self::key), [`span`](Self::span) and [`value`](Self::value).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> Attribute<S2> {
        Attribute {
            key: self.key.map_span(map),
            span: map(self.span),
            value: self.value.map(|v| v.map_span(map)),
        }
    }
}

/// A value of an [`Attribute`], which is a (mostly) arbitrary string.
///
/// This struct additionally stores the [`Span`] of the value.
#[derive(PartialEq, Clone, Debug)]
pub struct AttributeValue<S: Span = FullSpan> {
    /// The value associated with the key of the attribute
    pub value: String,
    /// The span covering the value
    pub span: S,
}

impl<S: Span> AttributeValue<S> {
    /// Maps every [`Span`] of this `AttributeValue` according to mapping function `map`.
    ///
    /// The new spans are of type `S2`, which may be different from the original span type `S`.
    /// `map` is applied to [`span`](Self::span).
    ///
    /// Usage is analogous to [`Expression::map_span()`]. Refer to its documentation for details and
    /// examples.
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> AttributeValue<S2> {
        AttributeValue {
            value: self.value,
            span: map(self.span),
        }
    }
}
