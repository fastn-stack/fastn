/// Trait for types that collect diagnostics and metadata during parsing.
///
/// The `Collector` trait is implemented by types that accumulate parsing results,
/// including errors, warnings, and comments. This allows the scanner to report
/// issues and track source annotations as it processes input.
pub trait Collector {
    /// Adds an error with its location to the collection.
    fn add_error(&mut self, span: fastn_section::Span, error: fastn_section::Error);

    /// Adds a warning with its location to the collection.
    fn add_warning(&mut self, span: fastn_section::Span, warning: fastn_section::Warning);

    /// Records the location of a comment in the source.
    fn add_comment(&mut self, span: fastn_section::Span);
}

/// A character-based scanner for parsing fastn source text.
///
/// The scanner provides methods for:
/// - Character-level navigation (peek, pop, reset)
/// - Token matching and consumption
/// - Whitespace and comment handling
/// - Span tracking for error reporting
///
/// It operates on UTF-8 text and correctly handles multi-byte characters.
/// The scanner maintains both character position and byte position for
/// accurate span creation.
#[derive(Debug)]
pub struct Scanner<'input, T: Collector> {
    input: &'input arcstr::ArcStr,
    pub module: fastn_section::Module,
    chars: std::iter::Peekable<std::str::CharIndices<'input>>,
    /// index is byte position in the input
    index: usize,
    #[expect(unused)]
    fuel: fastn_section::Fuel,
    pub output: T,
}

/// A saved position in the scanner that can be used for backtracking.
///
/// `Index` captures both the byte position and the character iterator state,
/// allowing the scanner to restore to a previous position when parsing fails
/// or when trying alternative parse paths.
#[derive(Clone)]
pub struct Index<'input> {
    index: usize,
    chars: std::iter::Peekable<std::str::CharIndices<'input>>,
}

impl<'input> PartialEq for Index<'input> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<'input, T: Collector> Scanner<'input, T> {
    pub fn add_error(&mut self, span: fastn_section::Span, error: fastn_section::Error) {
        self.output.add_error(span, error)
    }

    pub fn add_warning(&mut self, span: fastn_section::Span, warning: fastn_section::Warning) {
        self.output.add_warning(span, warning)
    }

    pub fn add_comment(&mut self, span: fastn_section::Span) {
        self.output.add_comment(span)
    }

    /// Creates a new scanner for the given input text.
    ///
    /// # Parameters
    /// - `input`: The source text to scan
    /// - `fuel`: Resource limit tracker (currently unused)
    /// - `module`: The module context for span creation
    /// - `t`: The collector for errors, warnings, and comments
    ///
    /// # Panics
    /// Panics if the input is larger than 10MB.
    pub fn new(
        input: &'input arcstr::ArcStr,
        fuel: fastn_section::Fuel,
        module: fastn_section::Module,
        t: T,
    ) -> Scanner<'input, T> {
        assert!(input.len() < 10_000_000); // can't unresolved > 10MB file
        Scanner {
            chars: input.char_indices().peekable(),
            input,
            fuel,
            index: 0,
            module,
            output: t,
        }
    }

    /// Creates a span from a saved index to the current position.
    ///
    /// This is commonly used to capture the text consumed during parsing.
    pub fn span(&self, start: Index) -> fastn_section::Span {
        fastn_section::Span {
            inner: self.input.substr(start.index..self.index),
            module: self.module,
        }
    }

    /// Creates a span between two saved indices.
    ///
    /// Useful for creating spans that don't end at the current position.
    pub fn span_range(&self, start: Index, end: Index) -> fastn_section::Span {
        fastn_section::Span {
            inner: self.input.substr(start.index..end.index),
            module: self.module,
        }
    }

    /// Consumes characters while the predicate returns true.
    ///
    /// Returns a span of the consumed text, or `None` if no characters matched.
    pub fn take_while<F: Fn(char) -> bool>(&mut self, f: F) -> Option<fastn_section::Span> {
        let start = self.index();
        while let Some(c) = self.peek() {
            if !f(c) {
                break;
            }
            self.pop();
        }

        if self.index == start.index {
            return None;
        }

        Some(self.span(start))
    }

    /// Saves the current position for potential backtracking.
    ///
    /// The returned `Index` can be passed to `reset()` to restore the scanner
    /// to this position if parsing fails.
    pub fn index(&self) -> Index<'input> {
        Index {
            index: self.index,
            chars: self.chars.clone(),
        }
    }

    /// Restores the scanner to a previously saved position.
    ///
    /// This is used for backtracking when a parse attempt fails and
    /// an alternative needs to be tried.
    pub fn reset(&mut self, index: &Index<'input>) {
        self.index = index.index;
        self.chars = index.chars.clone();
    }

    /// Looks at the next character without consuming it.
    ///
    /// Returns `None` if at the end of input.
    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|v| v.1)
    }

    /// Consumes and returns the next character.
    ///
    /// Updates the scanner's position by the character's byte length.
    /// Returns `None` if at the end of input.
    pub fn pop(&mut self) -> Option<char> {
        let (idx, c) = self.chars.next()?;
        // Update the index by the byte length of the character
        self.index = idx + c.len_utf8();
        Some(c)
    }

    /// Consumes a specific character if it's next in the input.
    ///
    /// Returns `true` if the character was consumed, `false` otherwise.
    pub fn take(&mut self, t: char) -> bool {
        if self.peek() == Some(t) {
            self.pop();
            true
        } else {
            false
        }
    }

    /// Skips spaces and tabs (but not newlines).
    ///
    /// This is used when horizontal whitespace should be ignored but
    /// line breaks are significant.
    pub fn skip_spaces(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.pop();
                continue;
            }
            break;
        }
    }

    /// Skips newline characters.
    ///
    /// Consumes any sequence of '\n' characters.
    pub fn skip_new_lines(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                self.pop();
                continue;
            }
            break;
        }
    }

    /// Skips all whitespace including spaces, tabs, newlines, and comments.
    ///
    /// This method repeatedly skips:
    /// - Spaces and tabs (via `skip_spaces`)
    /// - Newlines (via `skip_new_lines`)
    /// - Comments starting with `;;` (via `skip_comment`)
    ///
    /// It continues until no more whitespace or comments can be skipped.
    /// This is useful for parsing constructs that allow arbitrary whitespace
    /// and comments between tokens, such as generic type parameters.
    ///
    /// # Example
    /// ```text
    /// foo<
    ///   ;; This comment is skipped
    ///   bar
    ///   ;; So is this one
    ///   <
    ///     k>
    /// >
    /// ```
    pub fn skip_all_whitespace(&mut self) {
        // Skip all whitespace including spaces, tabs, newlines, and comments
        // We need to loop because these might be interleaved
        loop {
            let start_index = self.index();
            self.skip_spaces();
            self.skip_new_lines();
            self.skip_comment(); // Skip ;; comments
            // If we didn't advance, we're done
            if self.index() == start_index {
                break;
            }
        }
    }

    /// Skips a line comment if the scanner is positioned at one.
    ///
    /// Comments in fastn start with `;;` and continue until the end of the line.
    /// The newline character itself is not consumed.
    ///
    /// Returns `true` if a comment was found and skipped, `false` otherwise.
    ///
    /// # Example
    /// ```text
    /// ;; This is a comment
    /// foo<
    ///   ;; Comments can appear in generic parameters
    ///   bar
    /// >
    /// ```
    ///
    /// If the scanner is not at a comment (doesn't start with `;;`), the scanner
    /// position remains unchanged.
    pub fn skip_comment(&mut self) -> bool {
        // Check if we're at the start of a comment
        let start = self.index();
        if self.peek() != Some(';') {
            return false;
        }
        self.pop();
        if self.peek() != Some(';') {
            // Not a comment, restore position
            self.reset(&start);
            return false;
        }
        self.pop();

        // Skip until end of line
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.pop();
        }
        true
    }

    /// Consumes characters until a specific character or newline is found.
    ///
    /// This is commonly used for parsing header values that end at newline
    /// or when an expression marker (like '{') is encountered.
    pub fn take_till_char_or_end_of_line(&mut self, t: char) -> Option<fastn_section::Span> {
        self.take_while(|c| c != t && c != '\n')
    }

    /// Returns the remaining unparsed input (for testing).
    ///
    /// This method verifies that the character-based and byte-based
    /// remaining text are consistent.
    #[cfg(test)]
    pub fn remaining(&self) -> &str {
        let char_remaining = self.chars.clone().map(|c| c.1).collect::<String>();
        let str_remaining = &self.input[self.index..];

        assert_eq!(
            char_remaining, str_remaining,
            "Character-based and byte-based remaining text do not match"
        );

        str_remaining
    }

    /// Tries to match one of several string tokens.
    ///
    /// Returns the first matching token, or `None` if none match.
    /// This is useful for parsing keywords like "public", "private", etc.
    pub fn one_of(&mut self, choices: &[&'static str]) -> Option<&'static str> {
        #[allow(clippy::manual_find)]
        // clippy wants us to use this:
        //
        // ```rs
        // choices
        //     .iter()
        //     .find(|&choice| self.token(choice).is_some())
        //     .copied();
        // ```
        //
        // but this is clearer:
        for choice in choices {
            if self.token(choice).is_some() {
                return Some(choice);
            }
        }
        None
    }

    /// Tries to match and consume a specific string token.
    ///
    /// Returns a span of the matched token if successful, or `None` if the
    /// token doesn't match at the current position. On failure, the scanner
    /// position is unchanged (automatic backtracking).
    pub fn token(&mut self, t: &'static str) -> Option<fastn_section::Span> {
        let start = self.index();
        for char in t.chars() {
            if self.peek() != Some(char) {
                self.reset(&start);
                return None;
            }
            self.pop();
        }

        Some(self.span(start))
    }
}
