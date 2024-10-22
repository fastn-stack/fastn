pub struct Scanner {
    // source: String,
    tokens: Vec<(fastn_p1::Token, fastn_p1::Span)>,
    index: usize,
    pub output: fastn_p1::ParseOutput,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        use logos::Logos;
        Scanner {
            tokens: fastn_p1::Token::lexer(&source)
                .spanned()
                .map(|(r, span)| (r.unwrap(), span))
                .collect(),
            index: 0,
            output: fastn_p1::ParseOutput::default(),
        }
    }

    pub fn is_done(&self) -> bool {
        self.index >= self.tokens.len()
    }

    pub fn peek(&self) -> Option<(fastn_p1::Token, fastn_p1::Span)> {
        self.tokens.get(self.index).map(|v| v.to_owned())
    }

    pub fn pop(&mut self) -> Option<(fastn_p1::Token, fastn_p1::Span)> {
        match self.tokens.get(self.index) {
            Some(t) => {
                self.index += 1;
                Some(t.to_owned())
            }
            None => None,
        }
    }
}
