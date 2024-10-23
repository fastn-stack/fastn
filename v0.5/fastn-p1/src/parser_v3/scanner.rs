pub struct Scanner {
    // source: String,
    tokens: Vec<(fastn_p1::Token, fastn_p1::Span)>,
    index: usize,
    pub output: fastn_p1::ParseOutput,
}

impl Scanner {
    pub fn new(name: &str, source: &str) -> Scanner {
        use logos::Logos;
        Scanner {
            tokens: fastn_p1::Token::lexer(source)
                .spanned()
                .map(|(r, span)| (dbg!(r.unwrap()), span))
                .collect(),
            index: 0,
            output: fastn_p1::ParseOutput {
                doc_name: name.to_string(),
                ..Default::default()
            },
        }
    }

    pub fn is_done(&self) -> bool {
        self.index >= self.tokens.len()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn reset(&mut self, index: usize) {
        self.index = index;
    }

    pub fn next_is(&self, token: fastn_p1::Token) -> bool {
        self.peek().map(|(t, _)| t == token).unwrap_or(false)
    }

    pub fn peek(&self) -> Option<(fastn_p1::Token, fastn_p1::Span)> {
        self.tokens.get(self.index).map(|v| v.to_owned())
    }

    pub fn space_till(&mut self, t: fastn_p1::Token) -> Option<fastn_p1::Span> {
        self.take(fastn_p1::Token::Space)?;
        self.take(t)
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

    pub fn take(&mut self, token: fastn_p1::Token) -> Option<fastn_p1::Span> {
        if let Some((t, s)) = self.peek() {
            if t == token {
                self.pop();
                return Some(s);
            }
        }
        None
    }

    pub fn take_consecutive(&mut self, token: fastn_p1::Token) -> Option<fastn_p1::Span> {
        let mut span = self.take(token)?;
        while let Some(s) = self.take(token) {
            fastn_p1::parser_v3::utils::extend_range(&mut span, s);
        }
        Some(span)
    }

    pub fn one_of(
        &mut self,
        tokens: &[fastn_p1::Token],
    ) -> Option<(fastn_p1::Token, fastn_p1::Span)> {
        if let Some((t, _)) = self.peek() {
            if tokens.contains(&t) {
                return self.pop();
            }
        }
        None
    }

    // eats up all the comments and empty lines till first non-comment, returns if we are done
    pub fn gobble(&mut self) -> bool {
        // TODO: we can reduce the number of items here by using take_consecutive for comments
        //       and newlines
        while let Some((token, span)) = self.one_of(&[
            fastn_p1::Token::CommentLine,
            fastn_p1::Token::EmptyLine,
            fastn_p1::Token::Space,
        ]) {
            if token == fastn_p1::Token::CommentLine {
                self.output.insert_comment(span);
            }
        }
        self.is_done()
    }

    // eats up all the comments till first non-comment, returns if we are done
    pub fn gobble_comments(&mut self) -> bool {
        // TODO: we can reduce the number of items here by using take_consecutive for comments
        while let Some(span) = self.take(fastn_p1::Token::CommentLine) {
            self.output.insert_comment(span);
        }
        self.is_done()
    }

    pub fn add_error(&mut self, error: fastn_p1::SingleError, span: fastn_p1::Span) {
        self.output.items.push(fastn_p1::parser_v3::utils::spanned(
            fastn_p1::Item::Error(error),
            span,
        ));
    }

    pub fn add_errors(&mut self, errors: &mut Vec<fastn_p1::Spanned<fastn_p1::SingleError>>) {
        self.output
            .items
            .extend(errors.drain(..).map(|v| v.map(fastn_p1::Item::Error)));
    }
}
