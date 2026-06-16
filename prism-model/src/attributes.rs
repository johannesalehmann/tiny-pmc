/// Collection of attributes. Each attribute either has form `key` or `key=value`.
///
/// Attributes are not a feature of the PRISM language. They are present here to support use-cases
/// that need additional information (e.g. program repair, where you want to mark which components
/// should be repaired).
#[derive(PartialEq, Clone, Debug)]
pub struct Attributes {
    attributes: Vec<Attribute>,
}

impl Attributes {
    /// Constructs and empty set of attributes.
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    /// Constructs a set of attributes with the given content.
    pub fn with_attributes(attributes: Vec<Attribute>) -> Self {
        Self { attributes }
    }

    /// Adds an attribute to the collection.
    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.push(attribute);
    }

    /// Returns a reference to the attribute with the given key, if it exists.
    ///
    /// This function has O(n) complexity -- if the collection contains a lot of attributes, it
    /// may be slow.
    pub fn get(&self, key: &str) -> Option<&Attribute> {
        for attribute in &self.attributes {
            if attribute.key == key {
                return Some(attribute);
            }
        }
        None
    }

    /// Returns a mutable reference to the attribute with the given key, if it exists.
    ///
    /// This function has O(n) complexity -- if the collection contains a lot of attributes, it
    /// may be slow.
    pub fn get_mut(&self, key: &str) -> Option<&Attribute> {
        for attribute in &self.attributes {
            if attribute.key == key {
                return Some(attribute);
            }
        }
        None
    }
}

/// An attribute, consisting of a key and an optional value.
#[derive(PartialEq, Clone, Debug)]
pub struct Attribute {
    /// The key of the attribute.
    pub key: String,

    /// The name of the attribute.
    pub value: Option<String>,
}
