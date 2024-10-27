#[derive(Default)]
pub struct Scanner {
    pub tokens: Vec<char>,
    pub size: usize,
    index: usize,
    s_index: usize,
    ticks: std::cell::RefCell<usize>,
    pub output: fastn_p1::ParseOutput,
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

    #[allow(clippy::misnamed_getters)]
    pub fn index(&self) -> usize {
        self.s_index
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
                break;
            }
            self.pop();
        }
    }

    #[cfg(test)]
    pub fn remaining(&self) -> String {
        let mut s = String::new();
        for c in &self.tokens[self.index..] {
            s.push(*c);
        }
        s
    }
}
