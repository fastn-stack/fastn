#[derive(Default)]
pub struct Scanner {
    pub tokens: Vec<char>,
    pub size: usize,
    index: usize,
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
            Some(c)
        } else {
            None
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

    pub fn identifier(&mut self) -> Option<fastn_p1::Span> {
        let first = self.peek()?;
        // the first character should be is_alphabetic or `_`
        if !first.is_alphabetic() && first != '_' {
            return None;
        }

        let start = self.index;
        self.pop();

        // later characters should be is_alphanumeric or `_` or `-`
        while let Some(c) = self.peek() {
            if !c.is_alphanumeric() && c != '_' && c != '-' {
                break;
            }
            self.pop();
        }

        Some(fastn_p1::Span {
            start,
            end: self.index,
        })
    }
}
