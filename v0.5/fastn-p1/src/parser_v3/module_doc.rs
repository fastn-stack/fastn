pub fn module_doc(scanner: &mut fastn_p1::parser_v3::scanner::Scanner) -> bool {
    let mut module_doc: Option<fastn_p1::Span> = None;
    while let Some(v) = scanner.peek() {
        match v {
            (fastn_p1::Token::DocCommentLine, span) => {
                scanner.pop();

                if let Some(s) = &mut module_doc {
                    fastn_p1::parser_v3::utils::extend_range(s, span)
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
