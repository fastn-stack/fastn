pub trait ECey {
    fn add_error(&mut self, span: fastn_section::Span, message: fastn_section::Error);
    fn add_comment(&mut self, span: fastn_section::Span);
}

#[derive(Debug)]
pub struct Scanner<'input, T: ECey> {
    input: &'input arcstr::ArcStr,
    pub module: fastn_section::Module,
    chars: std::iter::Peekable<std::str::CharIndices<'input>>,
    /// index is byte position in the input
    index: usize,
    #[expect(unused)]
    fuel: fastn_section::Fuel,
    pub output: T,
}

pub struct Index<'input> {
    index: usize,
    chars: std::iter::Peekable<std::str::CharIndices<'input>>,
}

impl<'input> PartialEq for Index<'input> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<'input, T: ECey> Scanner<'input, T> {
    pub fn add_error(&mut self, span: fastn_section::Span, message: fastn_section::Error) {
        self.output.add_error(span, message)
    }

    pub fn add_comment(&mut self, span: fastn_section::Span) {
        self.output.add_comment(span)
    }

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

    pub fn span(&self, start: Index) -> fastn_section::Span {
        fastn_section::Span {
            inner: self.input.substr(start.index..self.index),
            module: self.module,
        }
    }

    pub fn span_range(&self, start: Index, end: Index) -> fastn_section::Span {
        fastn_section::Span {
            inner: self.input.substr(start.index..end.index),
            module: self.module,
        }
    }

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

    pub fn index(&self) -> Index<'input> {
        Index {
            index: self.index,
            chars: self.chars.clone(),
        }
    }

    pub fn reset(&mut self, index: Index<'input>) {
        self.index = index.index;
        self.chars = index.chars;
    }

    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|v| v.1)
    }

    pub fn pop(&mut self) -> Option<char> {
        let (idx, c) = self.chars.next()?;
        // Update the index by the byte length of the character
        self.index = idx + c.len_utf8();
        Some(c)
    }

    pub fn take(&mut self, t: char) -> bool {
        if self.peek() == Some(t) {
            self.pop();
            true
        } else {
            false
        }
    }

    pub fn skip_spaces(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.pop();
                continue;
            }
            break;
        }
    }

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
            self.reset(start);
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

    pub fn take_till_char_or_end_of_line(&mut self, t: char) -> Option<fastn_section::Span> {
        self.take_while(|c| c != t && c != '\n')
    }

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

    // returns the span from current position to the end of token
    pub fn token(&mut self, t: &'static str) -> Option<fastn_section::Span> {
        let start = self.index();
        for char in t.chars() {
            if self.peek() != Some(char) {
                self.reset(start);
                return None;
            }
            self.pop();
        }

        Some(self.span(start))
    }
}
