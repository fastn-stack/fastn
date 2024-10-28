#[derive(Default, Debug)]
pub struct Scanner {
    pub tokens: Vec<char>,
    pub size: usize,
    index: usize,
    s_index: usize,
    fuel: fastn_p1::Fuel,
    pub output: fastn_p1::ParseOutput,
}

pub struct Index {
    chars: usize,
    bytes: usize,
}

impl Scanner {
    pub fn new(source: &str, fuel: fastn_p1::Fuel) -> Scanner {
        assert!(source.len() < 10_000_000); // can't parse > 10MB file
        let tokens: Vec<_> = source.chars().collect();
        Scanner {
            size: tokens.len(),
            tokens,
            fuel,
            ..Default::default()
        }
    }

    pub fn span(&self, start: Index) -> fastn_p1::Span {
        fastn_p1::Span {
            start: start.bytes,
            end: self.s_index,
        }
    }

    pub fn index(&self) -> Index {
        Index {
            bytes: self.s_index,
            chars: self.index,
        }
    }

    /// Converts a given character count from the current index into the equivalent byte count.
    fn char_count_to_byte_count(&self, char_count: usize) -> usize {
        self.tokens[self.index..self.index + char_count]
            .iter()
            .map(|c| c.len_utf8()) // Get the byte length of each character
            .sum() // Sum up the byte lengths
    }

    /// Advances the scanner's character and byte indices by a specified number of characters.
    fn increment_index_by(&mut self, count: usize) {
        self.s_index += self.char_count_to_byte_count(count);
        self.index += count;
    }

    /// Creates a `Span` covering the next `count` characters and advances the scanner's position.
    fn span_for_chars_with_advance(&mut self, count: usize) -> fastn_p1::Span {
        let start = self.s_index;
        self.increment_index_by(count);

        fastn_p1::Span {
            start,
            end: self.s_index,
        }
    }

    pub fn reset(&mut self, index: Index) {
        self.s_index = index.bytes;
        self.index = index.chars;
    }

    pub fn peek(&self) -> Option<char> {
        if self.index < self.size {
            Some(self.tokens[self.index])
        } else {
            None
        }
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.index < self.size {
            let c = self.tokens[self.index];
            self.increment_index_by(1);
            Some(c)
        } else {
            None
        }
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

    pub fn read_till_char_or_end_of_line(&mut self, t: char) -> Option<fastn_p1::Span> {
        let mut count = 0;
        while let Some(c) = self.tokens.get(self.index + count) {
            if *c == t || *c == '\n' {
                break;
            }
            count += 1;
        }
        if count == 0 {
            return None;
        }

        Some(self.span_for_chars_with_advance(count))
    }

    #[cfg(test)]
    pub fn remaining(&self) -> String {
        let char_remaining = self.tokens[self.index..].iter().collect::<String>();
        let byte_remaining = self.tokens.iter().collect::<String>()[self.s_index..].to_string();

        assert_eq!(
            char_remaining, byte_remaining,
            "Character-based and byte-based remaining text do not match"
        );

        char_remaining
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
    pub fn token(&mut self, t: &'static str) -> Option<fastn_p1::Span> {
        let mut count = 0;
        for char in t.chars() {
            assert!(char.is_ascii()); // we are assuming this is ascii string
            if (self.index + count < self.size) && (char != self.tokens[self.index + count]) {
                return None;
            }

            count += 1;
        }

        Some(self.span_for_chars_with_advance(count))
    }
}
