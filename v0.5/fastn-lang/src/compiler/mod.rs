/// this is our main compiler module
///
/// it should be called with a parsed document, and it returns generated JS.
///
/// on success, we return the JS, and list of warnings, and on error, we return the list of
/// diagnostics, which is an enum containing warning and error.
///
/// earlier we had strict mode here, but to simplify things, now we let the caller convert non-empty
/// warnings from OK part as error, and discard the generated JS.
pub async fn compile(
    _ds: &mut Box<dyn fastn_lang::DS>,
    _document: &fastn_section::Document,
) -> Result<(String, Vec<fastn_section::Warning>), Vec<fastn_section::Diagnostic>> {
    todo!()
}
