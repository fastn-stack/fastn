impl fastn_p1::ParseOutput {
    pub fn new(_name: &str, source: &str) -> fastn_p1::ParseOutput {
        let mut scanner = Scanner::new(source.to_string());

        if source_doc(&mut scanner) {
            return scanner.output;
        }

        scanner.output
    }
}

fn source_doc(scanner: &mut Scanner) -> bool {
    let mut module_doc: Option<fastn_p1::Span> = None;
    while let Some(v) = scanner.peek() {
        match v {
            (fastn_p1::Token::DocCommentLine, span) => {
                scanner.pop();

                if let Some(s) = &mut module_doc {
                    extend_range(s, span)
                } else {
                    module_doc = Some(span);
                }
            }
            (fastn_p1::Token::CommentLine, span) => {
                scanner.pop();
                scanner.output.items.push(fastn_p1::Spanned {
                    span,
                    value: fastn_p1::Item::Comment,
                });
                // comments at the beginning of the file, before the doc comment, is allowed, e.g.,
                // software license etc. are often put there.
                //
                // if we have never read any doc_comment line, this will be non, and we keep looking
                // for possibly nore comment. if we read any doc_comment line, we are done.
                if module_doc.is_some() {
                    break;
                }
            }
            // if we find anything else, we are done collecting module_doc
            _ => break,
        }
    }

    scanner.output.module_doc = module_doc;
    scanner.is_done()
}

fn extend_range(a: &mut fastn_p1::Span, b: fastn_p1::Span) {
    assert_eq!(a.end, b.start);
    a.end = b.end;
}

struct Scanner {
    // source: String,
    tokens: Vec<(fastn_p1::Token, fastn_p1::Span)>,
    index: usize,
    output: fastn_p1::ParseOutput,
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
