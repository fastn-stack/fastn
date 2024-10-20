#[derive(logos::Logos, Debug, PartialEq, Clone)]
enum Token {
    #[token(":")]
    Colon,

    #[token("$")]
    Dollar,

    #[token("$$")]
    DoubleDollar,

    #[token("${")]
    DollarCurly,

    #[token("$${")]
    DoubleDollarCurly,

    #[token("{")]
    Curly,

    #[token("}")]
    CurlyClose,

    #[token("(")]
    Paren,

    #[token(")")]
    ParenClose,

    #[token("()")]
    FnMarker,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("=")]
    Assignment,

    #[token("==")]
    Equal,

    #[token(">=")]
    Gte,

    #[token("<=")]
    Lte,

    #[token("!=")]
    NotEqual,

    #[token("!")]
    Not,

    #[token("*")]
    Cross,

    #[token("/")]
    Slash,

    #[token("as")]
    As,

    #[token("^")]
    Caret,

    #[token("$[")]
    DollarSquare,

    #[token("[")]
    Square,

    #[token("]")]
    SquareClose,

    #[token("\n")]
    NewLine,

    #[regex("[\t ]+")]
    Space,

    #[regex("<")]
    Angle,

    #[regex(">")]
    AngleClose,

    #[token("--")]
    DashDash,

    #[token(";;")]
    SemiSemi,

    #[token(";-;")]
    SemiDashSemi,

    #[token("component")]
    Component,

    #[token("record")]
    Record,

    #[token("import")]
    Import,

    #[token("let")]
    Let,

    #[token("public")]
    Public,

    #[token("private")]
    Private,

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", priority = 3)]
    Number,

    #[regex(r"[\w]+")]
    Word,

    #[token(".")]
    Period,

    #[token(",")]
    Comma,
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        use logos::Logos;
        let source = include_str!("../t/002-tutorial.ftd");
        assert_eq!(super::Token::lexer(source).spanned().clone().count(), 1372);
    }
}
