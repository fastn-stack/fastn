#[derive(Default, Debug)]
pub struct Scanner {
    pub tokens: Vec<char>,
    pub size: usize,
    index: usize,
    s_index: usize,
    fuel: fastn_p1::Fuel,
    pub output: fastn_p1::ParseOutput,
}

#[derive(Debug, PartialEq)]
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

    fn span(&self, start: Index) -> fastn_p1::Span {
        fastn_p1::Span {
            start: start.bytes,
            end: self.s_index,
        }
    }

    pub fn eat_while<F: Fn(char) -> bool>(&mut self, f: F) -> Option<fastn_p1::Span> {
        let start = self.index();
        while let Some(c) = self.peek() {
            if !f(c) {
                break;
            }
            self.pop();
        }

        if self.index() == start {
            return None;
        }

        Some(self.span(start))
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
        self.eat_while(|c| c != t && c != '\n')
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
        let start = self.index();
        for char in t.chars() {
            assert!(char.is_ascii()); // we are assuming this is ascii string
            if self.peek() != Some(char) {
                self.reset(start);
                return None;
            }
            self.pop();
        }

        Some(self.span(start))
    }
}
