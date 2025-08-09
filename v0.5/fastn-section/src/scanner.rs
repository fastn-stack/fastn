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
