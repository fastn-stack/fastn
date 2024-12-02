#[allow(clippy::too_many_arguments)]
pub fn arguments(
    arguments: &[fastn_resolved::Argument],
    caption: &mut fastn_unresolved::UR<Option<fastn_section::HeaderValue>, ()>,
    #[expect(clippy::ptr_arg)] _properties: &mut Vec<
        fastn_unresolved::UR<fastn_unresolved::Property, fastn_resolved::Property>,
    >,
    body: &mut fastn_unresolved::UR<Option<fastn_section::HeaderValue>, ()>,
    _children: &[fastn_unresolved::UR<
        fastn_unresolved::ComponentInvocation,
        fastn_resolved::ComponentInvocation,
    >],
    _definitions: &std::collections::HashMap<String, fastn_unresolved::URD>,
    _modules: &std::collections::HashMap<fastn_unresolved::Module, bool>,
    _arena: &mut fastn_unresolved::Arena,
    _output: &mut fastn_unresolved::resolver::Output,
) {
    caption_or_body(caption, true, arguments);
    caption_or_body(body, false, arguments);
}

fn caption_or_body(
    v: &mut fastn_unresolved::UR<Option<fastn_section::HeaderValue>, ()>,
    _is_caption: bool,
    arguments: &[fastn_resolved::Argument],
) {
    if let fastn_unresolved::UR::UnResolved(None) = v {
        *v = fastn_unresolved::UR::Resolved(())
    }
    if let fastn_unresolved::UR::UnResolved(Some(_v)) = v {
        // see if any of the arguments are of type caption.
        // assume there is only one such argument, because otherwise arguments would have failed
        // to resolve
        match arguments.iter().find(|v| v.is_caption()) {
            Some(_v) => todo!(),
            None => *v = fastn_unresolved::UR::Invalid(fastn_section::Error::UnexpectedCaption),
        }
    }
}
