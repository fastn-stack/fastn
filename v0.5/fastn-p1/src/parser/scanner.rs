#[derive(Default, Debug)]
pub struct Scanner {
    pub tokens: Vec<char>,
    pub size: usize,
    index: usize,
    s_index: usize,
    ticks: std::cell::RefCell<usize>,
    pub output: fastn_p1::ParseOutput,
}

pub struct Index {
    chars: usize,
    bytes: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        let tokens: Vec<_> = source.chars().collect();
        Scanner {
            size: tokens.len(),
            tokens,
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

    // #[cfg(test)]
    pub fn remaining(&self) -> String {
        let mut s = String::new();
        for c in &self.tokens[self.index..] {
            s.push(*c);
        }
        s
    }

    pub fn one_of(&mut self, choices: &[&'static str]) -> Option<&'static str> {
        'outer: for choice in choices {
            let mut count = 0;
            for char in choice.chars() {
                assert!(char.is_ascii()); // we are assuming this is ascii string
                if char != self.tokens[self.index + count] {
                    continue 'outer;
                }
                count += 1
            }
            self.index += count;
            self.s_index = self.index;
            return Some(choice);
        }
        None
    }
}
