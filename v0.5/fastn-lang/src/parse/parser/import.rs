pub(super) fn import(
    _source: &str,
    section: fastn_section::token::Section,
    _document: &mut fastn_section::parse::Document,
) {
    if let Some(_kind) = section.init.name.kind {
        // document.errors.push(fastn_section::Error::ImportCantHaveType);
        todo!()
    }
    // section.name must be exactly import.
    // section.caption must be single text block, parsable as a module-name.
    //       module-name must be internally able to handle aliasing.
    // only two headers allowed: exports and exposing, parse them.
    // ensure there are no children or body.
    todo!()
}
