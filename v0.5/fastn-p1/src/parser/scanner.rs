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
            self.index += 1;
            // increment s_index by size of c
            self.s_index += c.len_utf8();
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
        let span = fastn_p1::Span {
            start: self.s_index,
            end: self.s_index + count,
        };
        self.index += count;
        self.s_index += count;
        Some(span)
    }

    #[cfg(test)]
    pub fn remaining(&self) -> String {
        let mut s = String::new();
        for c in &self.tokens[self.index..] {
            s.push(*c);
        }
        s
    }

    #[cfg(test)]
    pub fn s_remaining(&self) -> String {
        let token: String = self.tokens.iter().collect();
        token[self.s_index..].to_string()
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
        // Get the length of the token to match
        let token_len = t.chars().count();

        // Ensure that we have enough characters left in the source to match the token
        if self.index + token_len > self.size {
            return None;
        }

        for (index, char) in t.chars().enumerate() {
            assert!(char.is_ascii()); // we are assuming this is ascii string
            if char != self.tokens[self.index + index] {
                return None;
            }
        }

        // increment both index and s_index by size of token, since token is ascii string so both
        // are incremented by the token length
        self.index += token_len;
        self.s_index += token_len;

        Some(fastn_p1::Span {
            start: self.s_index - token_len,
            end: self.s_index,
        })
    }
}
