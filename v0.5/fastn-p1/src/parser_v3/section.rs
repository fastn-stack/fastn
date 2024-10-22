pub fn section(scanner: &mut fastn_p1::parser_v3::scanner::Scanner) -> bool {
    scanner.gobble();

    // we will assume everything is an error until proven otherwise.
    //
    // when we actually encounter error, we will return all accumulated errors, else we will
    // drop them.
    let mut errors: Vec<fastn_p1::SingleError> = vec![];

    // section can start with doc comment, let's fetch it
    let doc_comment = scanner.take_consecutive(fastn_p1::Token::DocCommentLine);
    if let Some(span) = doc_comment {
        errors.push(fastn_p1::SingleError::UnexpectedDocComment(span));
    }

    // next must come `--`, if not we skip the line
    let dashdash = match scanner.take(fastn_p1::Token::DashDash) {
        Some(v) => v,
        None => {
            // we have to advance the cursor till the next line: only
            // EmptyLine, DocCommentLine and CommentLine contain newline, everything else
            // should be gobbled up as text, and added as UnwantedTextFound error

            // errors.push(fastn_p1::SingleError::UnwantedTextFound());
            scanner.enque_errors(errors);
            return false;
        }
    };

    // we capture any number of space, then dash-dash, then exactly one space, we need to
    // record just the range for dash-dash for syntax highlighting purpose
    fastn_p1::parser_v3::utils::subspan_from_end(dashdash, 3, 1);

    scanner.is_done()
}
