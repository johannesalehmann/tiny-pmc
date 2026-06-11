use std::ops::Range;

/// Provides a mapping between spans (which store locations as offsets from the beginning of the
/// document) and lines.
///
/// *Character offsets start at 0, line numbers start at 1.*
///
/// # Example
///
/// ```
/// # use prism_parser::{parse_model, CharacterToLineMap};
/// # use prism_model::Span;
/// let source =
/// r"mdp
/// module main
///     // Here dragons abound
/// endmodule
/// ";
/// let character_to_line = CharacterToLineMap::from_str(source);
/// let parsed = parse_model(source).expect("Error parsing model");
/// let main = parsed.modules.get(0).unwrap();
/// match main.span.range() {
///     Some(range) => {
///         assert_eq!(range, 4..52);
///         assert_eq!(character_to_line.get_lines(range), 2..5);
///     }
///     None => unreachable!(
/// )}
/// ```
///
/// This indicates that module `main` covers lines 2, 3 and 4.
#[derive(Clone, PartialEq, Debug)]
pub struct CharacterToLineMap {
    line_start_indices: Vec<usize>,
}

impl CharacterToLineMap {
    /// Constructs a character-to-line map without any entries.
    ///
    /// Use [`add_line()`](Self::add_line) to add line indices (or construct the
    /// entire map from a string using [`from_str()`](Self::from_str)).
    pub fn new() -> Self {
        Self {
            line_start_indices: Vec::new(),
        }
    }

    /// Constructs a character-to-line map for the given string.
    pub fn from_str<S: AsRef<str>>(source: S) -> Self {
        let mut res = Self::new();
        res.add_line(0);
        for (index, char) in source.as_ref().chars().enumerate() {
            if char == '\n' {
                res.add_line(index + 1);
            }
        }

        res
    }

    /// Adds a new line to the character-to-line map.
    ///
    /// `line_start_index` must be the index of the first character of the line, relative to the
    /// document's start.
    ///
    /// # Example
    ///
    /// ```
    /// use prism_parser::CharacterToLineMap;
    /// let source =
    /// r"line 1
    /// line 2
    /// line 3";
    ///
    /// let auto_map = CharacterToLineMap::from_str(source);
    ///
    /// let mut manual_map = CharacterToLineMap::new();
    /// manual_map.add_line(0);
    /// manual_map.add_line(7);
    /// manual_map.add_line(14);
    ///
    /// assert_eq!(auto_map, manual_map);
    /// ```
    pub fn add_line(&mut self, line_start_index: usize) {
        if let Some(&last_line_start_index) = self.line_start_indices.last() {
            if last_line_start_index >= line_start_index {
                panic!(
                    "Cannot add a line start index that is less or equal to the previous line start index."
                )
            }
        }
        self.line_start_indices.push(line_start_index);
    }

    /// Returns the line number that the character with given index is on. The first line of the
    /// document has index 1.
    pub fn get_line(&self, char_index: usize) -> usize {
        self.line_start_indices
            .partition_point(|line_start| *line_start <= char_index)
    }

    /// Returns the range of lines occupied by the characters in the given range.
    ///
    /// This is not the same as `get_line(range.start)..get_line(range.end)` -- handling the end
    /// of the range requires some care as it is an exclusive range.
    pub fn get_lines(&self, char_range: Range<usize>) -> Range<usize> {
        let start_range = self.get_line(char_range.start);
        let end_range = self.get_line((char_range.end - 1).max(char_range.start));
        start_range..end_range + 1
    }
}
