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

    pub fn one_of<'a>(&mut self, choices: &[&'a str]) -> Option<&'a str> {
        // TODO: store source reference, so we can use string matches instead of allocating
        //       potentially multiple vecs on every call to this function
        for choice in choices {
            let len = choice.chars().count();
            if self.index + len > self.size {
                continue;
            }
            if choice.chars().collect::<Vec<_>>() == self.tokens[self.index..self.index + len] {
                self.index += len;
                self.s_index += choice.len();
                return Some(choice);
            }
        }
        None
    }
}
