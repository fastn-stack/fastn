#[allow(clippy::too_many_arguments)]
pub fn arguments(
    arguments: &[fastn_resolved::Argument],
    caption: &mut fastn_unresolved::UR<Option<fastn_section::HeaderValue>, ()>,
    _properties: &mut Vec<
        fastn_unresolved::UR<fastn_unresolved::Property, fastn_resolved::Property>,
    >,
    _body: &mut fastn_unresolved::UR<Vec<fastn_section::Tes>, ()>,
    _children: &[fastn_unresolved::UR<
        fastn_unresolved::ComponentInvocation,
        fastn_resolved::ComponentInvocation,
    >],
    _definitions: &std::collections::HashMap<String, fastn_unresolved::URD>,
    _modules: &std::collections::HashMap<fastn_unresolved::Module, bool>,
    _arena: &mut fastn_unresolved::Arena,
    _output: &mut fastn_unresolved::resolver::Output,
) {
    if let fastn_unresolved::UR::UnResolved(None) = caption {
        *caption = fastn_unresolved::UR::Resolved(())
    }
    if let fastn_unresolved::UR::UnResolved(Some(_v)) = caption {
        // see if any of the arguments are of type caption.
        // assume there is only one such argument, because otherwise arguments would have failed
        // to resolve
        match arguments.iter().find(|v| v.is_caption()) {
            Some(_v) => todo!(),
            None => {
                *caption = fastn_unresolved::UR::Invalid(fastn_section::Error::UnexpectedCaption)
            }
        }
    }
}
