#[derive(logos::Logos, Debug, PartialEq, Clone, Copy)]
pub enum Token {
    #[token("\\;;")]
    EscapedComment,

    #[regex(r";;[^\n]*\n")]
    CommentLine,

    #[token("\\;-;")]
    EscapedDocComment,

    #[regex(r";-;[^\n]*\n")]
    DocCommentLine,

    #[regex("--")]
    DashDash,

    #[regex(r"[\w]+")]
    Word,

    #[regex("[\t ]+")]
    Space,

    #[regex(r"[ \t]*\n")]
    EmptyLine,

    #[regex(r"\([ \t]*\)")]
    FnMarker,

    #[token(":")]
    Colon,

    #[token("\\${")]
    EscapedDollarCurly,

    #[token("${")]
    DollarCurly,

    #[token("{")]
    Curly,

    #[token("}")]
    CurlyClose,

    #[token("$[")]
    DollarSquare,

    #[token("$$[")]
    DoubleDollarSquare,

    #[token("\\$[")]
    EscapedDollarSquare,

    #[token("\\$$[")]
    EscapedDoubleDollarSquare,

    #[token("[")]
    Square,

    #[token("]")]
    SquareClose,

    #[token("<")]
    Angle,

    #[token(">")]
    AngleClose,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        use logos::Logos;
        let source = include_str!("../t/002-tutorial.ftd");
        assert_eq!(
            dbg!(super::Token::lexer(source).spanned().collect::<Vec<_>>()).len(),
            585
        );
    }
}
