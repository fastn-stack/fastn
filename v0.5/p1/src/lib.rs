// #[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
// #[serde(default)]
pub struct Section<'a> {
    // this is the comment block we encountered before the section
    // pub pre_comment: Option<Sourced<&'a str>>,
    pub name: KindedName<'a>,
    pub caption: Option<Sourced<HeaderValue<'a>>>,
    pub headers: Vec<(KindedName<'a>, HeaderValue<'a>)>,
    pub body: Option<Sourced<HeaderValue<'a>>>,
    pub sub_sections: Vec<Section<'a>>,
}

pub struct KindedName<'a> {
    pub kind: Sourced<Option<&'a str>>,
    pub name: Sourced<&'a str>,
}

pub struct Sourced<T> {
    pub from: usize,
    pub to: usize,
    pub is_commented: bool,
    pub value: T,
}

pub type HeaderValue<'a> = Vec<Sourced<StringOrSection<'a>>>;

pub enum StringOrSection<'a> {
    // This is a `Cow<_>` because we will be escaping \{ and \} in the string, and also trimming
    // de-indenting the string
    String(std::borrow::Cow<'a, &'a str>),
    Section(Section<'a>),
}

pub enum ParseError {}

pub fn parse<'a>(
    _doc_name: &str,
    _source: &'a str,
) -> Result<Vec<Sourced<Section<'a>>>, ParseError> {
    todo!()
}
