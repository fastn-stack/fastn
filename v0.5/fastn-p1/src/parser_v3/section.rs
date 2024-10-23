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

    scanner.gobble_comments(); // this is because consecutive, non-empty line comments are allowed

    // the very next lines must be a section_header

    let _section_header = match section_header(scanner, potential_errors) {
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
    let dashdash = match scanner.space_till(fastn_p1::Token::DashDash) {
        Some(v) => v,
        None => {
            recover_from_error(scanner, potential_errors);
            return None;
        }
    };

    let _kinded_name = match kinded_name(scanner) {
        Some(v) => v,
        None => {
            recover_from_error(scanner, potential_errors);
            return None;
        }
    };

    None
}

/// kinded name contains an optional kind and a name.
///
/// newlines and comments are allowed inside `<>`.
///
/// the `<>` portion can nest.
///
/// the following declares a variable `foo` with type `list<string>`:
///
/// ```ftd
/// -- list<
///    ;; this is a comment
///    string
/// > foo: []
/// ```
fn kinded_name(scanner: &mut fastn_p1::parser_v3::scanner::Scanner) -> Option<fastn_p1::Span> {
    // try to read kind
    let _k = kind(scanner)?;
    // try to read name
    // if we find both kind and name, we return the span of both
    // if name is not found, see if kind is "simple" (without `<>`s), if so, it is the name
    todo!()
}

pub struct Kind {
    span: fastn_p1::Span,
    is_simple: bool,
}

/// kind returns just the kind. it can nest on `<>`. kind ends at space, unless what comes after
/// space is a `<`. or it ends at a `:`.
///
/// let's call the content inside `<>` as "angle text"
fn kind(scanner: &mut fastn_p1::parser_v3::scanner::Scanner) -> Option<Kind> {
    let word = scanner.space_till(fastn_p1::Token::Word)?;
    let mut is_simple = true;

    // check if the next item is a <
    if scanner.space_till(fastn_p1::Token::Angle).is_some() {
        is_simple = false;
        if !angle_text(scanner) {
            return None;
        }
    }

    Some(Kind {
        span: fastn_p1::Span {
            start: word.start,
            end: scanner.index(),
        },
        is_simple,
    })
}

fn angle_text(_scanner: &mut fastn_p1::parser_v3::scanner::Scanner) -> bool {
    todo!()
}

// this is error recovery for a section. if there is any error in the section, we skip till the
// beginning of next section, or till the end of the file.
fn recover_from_error(
    scanner: &mut fastn_p1::parser_v3::scanner::Scanner,
    potential_errors: &mut Vec<fastn_p1::Spanned<fastn_p1::SingleError>>,
) -> bool {
    // TODO: we have to advance the cursor till the next line: only EmptyLine, DocCommentLine and
    //       CommentLine contain newline, everything else should be gobbled up as text, and added
    //       as UnwantedTextFound error

    // errors.push(fastn_p1::SingleError::UnwantedTextFound());
    scanner.enqueue_errors(potential_errors);
    false
}
