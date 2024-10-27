pub fn section(
    scanner: &mut fastn_p1::parser_v3::scanner::Scanner,
    potential_errors: &mut Vec<fastn_p1::Spanned<fastn_p1::SingleError>>,
) {
    println!("reading section");

    scanner.gobble();

    // section can start with doc comment, let's fetch it
    let doc_comment = dbg!(scanner.take_consecutive(fastn_p1::Token::DocCommentLine));
    if let Some(span) = &doc_comment {
        potential_errors.push(fastn_p1::parser_v3::utils::spanned(
            fastn_p1::SingleError::UnexpectedDocComment,
            span.clone(),
        ));
    }

    scanner.gobble_comments(); // this is because consecutive, non-empty line comments are allowed

    eprintln!("here");
    // the very next lines must be a section_header
    let section_header = match section_header(scanner, potential_errors) {
        Some(v) => dbg!(v),
        None => {
            // we have to advance the cursor till the next line: only
            // EmptyLine, DocCommentLine and CommentLine contain newline, everything else
            println!("section_header not found");
            return;
        }
    };

    scanner.output.items.push(fastn_p1::Spanned {
        value: fastn_p1::Item::Section(Box::new(fastn_p1::Section {
            name: section_header.kinded_name,
            ..Default::default()
        })),
        span: fastn_p1::Span {
            start: section_header.dashdash.start,
            end: section_header.colon.end,
        },
    });
}

#[derive(Debug, Default)]
struct SectionHeader {
    dashdash: fastn_p1::Span,
    kinded_name: fastn_p1::KindedName,
    function_marker: Option<fastn_p1::Span>,
    colon: fastn_p1::Span,
}

// till colon
fn section_header(
    scanner: &mut fastn_p1::parser_v3::scanner::Scanner,
    potential_errors: &mut Vec<fastn_p1::Spanned<fastn_p1::SingleError>>,
) -> Option<SectionHeader> {
    println!("reading section_header");
    // next must come `--`, if not we skip the line
    let dashdash = match scanner.space_till(fastn_p1::Token::DashDash) {
        Some(v) => dbg!(v),
        None => {
            if scanner.is_done() {
                recover_from_error(scanner, potential_errors, None);
                return None;
            }
            println!("dashdash not found");
            recover_from_error(
                scanner,
                potential_errors,
                Some(scanner.current_spanned(fastn_p1::SingleError::DashDashNotFound)),
            );
            return None;
        }
    };

    let kinded_name = match kinded_name(scanner) {
        Some(v) => dbg!(v),
        None => {
            println!("kinded_name not found");
            recover_from_error(
                scanner,
                potential_errors,
                Some(scanner.current_spanned(fastn_p1::SingleError::KindedNameNotFound)),
            );
            return None;
        }
    };

    let function_marker = scanner.space_till(fastn_p1::Token::FunctionMarker);

    let colon = match scanner.space_till(fastn_p1::Token::Colon) {
        Some(v) => dbg!(v),
        None => {
            println!("colon not found");
            recover_from_error(
                scanner,
                potential_errors,
                Some(scanner.current_spanned(fastn_p1::SingleError::ColonNotFound)),
            );
            return None;
        }
    };

    Some(SectionHeader {
        dashdash,
        kinded_name,
        function_marker,
        colon,
    })
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
fn kinded_name(
    scanner: &mut fastn_p1::parser_v3::scanner::Scanner,
) -> Option<fastn_p1::KindedName> {
    println!("kinded_name");
    // try to read kind
    let mut k = kind(scanner)?;
    println!("kind: {k:?}");
    // try to read name
    match scanner.space_till(fastn_p1::Token::Word) {
        Some(v) => {
            // if we find both kind and name, we return the span of both
            Some(fastn_p1::KindedName {
                kind: Some(fastn_p1::Kind {
                    kind: k.span,
                    ..Default::default()
                }),
                name: fastn_p1::Span {
                    start: v.start,
                    end: scanner.index(),
                },
            })
        }
        None => {
            // if a name is not found, see if kind is "simple" (without `<>`s) if so, it is the name
            if k.is_simple {
                Some(fastn_p1::KindedName {
                    kind: None,
                    name: k.span,
                })
            } else {
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct Kind {
    span: fastn_p1::Span,
    is_simple: bool,
}

/// kind returns just the kind. it can nest on `<>`. kind ends at space, unless what comes after
/// space is a `<`. or it ends at a `:`.
///
/// let's call the content inside `<>` as "angle text"
fn kind(scanner: &mut fastn_p1::parser_v3::scanner::Scanner) -> Option<Kind> {
    println!("kind");
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

fn angle_text(scanner: &mut fastn_p1::parser_v3::scanner::Scanner) -> bool {
    println!("angle_text");
    // this is the inside of an angle text. it must be a word
    let start = scanner.index() - 1; // for EmptyAngleText error
    scanner.gobble();

    if scanner.take(fastn_p1::Token::Word).is_none() {
        if scanner.take(fastn_p1::Token::AngleClose).is_some() {
            scanner.add_error(
                fastn_p1::SingleError::EmptyAngleText,
                fastn_p1::Span {
                    start,
                    end: scanner.index(),
                },
            );
            return true;
        }

        return false;
    }

    scanner.gobble();

    // after the word we can find another `<`
    #[allow(clippy::collapsible_if)] // because makes code a little be more readable
    if scanner.take(fastn_p1::Token::Angle).is_some() {
        // so we recurse
        if !angle_text(scanner) {
            return false;
        }
    }

    // must end with `>`
    scanner.take(fastn_p1::Token::AngleClose).is_some()
}

// this is error recovery for a section. if there is any error in the section, we skip till the
// beginning of the next section, or till the end of the file.
fn recover_from_error(
    scanner: &mut fastn_p1::parser_v3::scanner::Scanner,
    potential_errors: &mut Vec<fastn_p1::Spanned<fastn_p1::SingleError>>,
    error: Option<fastn_p1::Spanned<fastn_p1::SingleError>>,
) {
    println!("recover_from_error");
    // TODO: we have to advance the cursor till the next line: only EmptyLine, DocCommentLine and
    //       CommentLine contain newline, everything else should be gobbled up as text, and added
    //       as UnwantedTextFound error

    // errors.push(fastn_p1::SingleError::UnwantedTextFound());
    scanner.add_errors(potential_errors);
    if let Some(error) = error {
        scanner.add_error(error.value, error.span);
    }

    scanner.skip_till(&[fastn_p1::Token::DashDash, fastn_p1::Token::DocCommentLine]);
}
