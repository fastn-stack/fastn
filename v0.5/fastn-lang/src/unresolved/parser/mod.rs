mod import;

fn parse(source: &str) -> fastn_lang::unresolved::Document {
    let mut document = fastn_lang::section::Document::parse(source);
    // for section in the document: guess the section and call the appropriate parse method.
    todo!()
}
