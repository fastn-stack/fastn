pub fn section(
    scanner: &mut fastn_p1::parser_v3::scanner::Scanner,
    potential_errors: &mut Vec<fastn_p1::Spanned<fastn_p1::SingleError>>,
) -> bool {
    scanner.gobble();

    // section can start with doc comment, let's fetch it
    let doc_comment = scanner.take_consecutive(fastn_p1::Token::DocCommentLine);
    if let Some(span) = doc_comment {
        potential_errors.push(fastn_p1::parser_v3::utils::spanned(
            fastn_p1::SingleError::UnexpectedDocComment,
            span,
        ));
    }

    let _section_line = match section_header(scanner, potential_errors) {
        Some(v) => v,
        None => {
            // we have to advance the cursor till the next line: only
            // EmptyLine, DocCommentLine and CommentLine contain newline, everything else
            return recover_from_error(scanner, potential_errors);
        }
    };

    scanner.is_done()
}

#[derive(Debug, Default)]
struct SectionHeader {
    dashdash: fastn_p1::Span,
    name: fastn_p1::Span,
    function: fastn_p1::Span,
    colon: fastn_p1::Span,
}

// till colon
fn section_header(
    scanner: &mut fastn_p1::parser_v3::scanner::Scanner,
    potential_errors: &mut Vec<fastn_p1::Spanned<fastn_p1::SingleError>>,
) -> Option<fastn_p1::Span> {
    // next must come `--`, if not we skip the line
    let dashdash = match scanner.take(fastn_p1::Token::DashDash) {
        Some(v) => v,
        None => {
            recover_from_error(scanner, potential_errors);
            return None;
        }
    };

    // we capture any number of space, then dash-dash, then exactly one space, we need to
    // record just the range for dash-dash for syntax highlighting purpose
    fastn_p1::parser_v3::utils::subspan_from_end(dashdash, 3, 1);

    None
}

// this is error recovery for a section. if there is any error in the section, we skip till the
// beginning of next section, or till the end of the file.
fn recover_from_error(
    scanner: &mut fastn_p1::parser_v3::scanner::Scanner,
    potential_errors: &mut Vec<fastn_p1::Spanned<fastn_p1::SingleError>>,
) -> bool {
    // we have to advance the cursor till the next line: only
    // EmptyLine, DocCommentLine and CommentLine contain newline, everything else
    // should be gobbled up as text, and added as UnwantedTextFound error

    // errors.push(fastn_p1::SingleError::UnwantedTextFound());
    scanner.enqueue_errors(potential_errors);
    false
}
