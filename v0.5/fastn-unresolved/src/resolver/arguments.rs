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
    is_caption: bool,
    arguments: &[fastn_resolved::Argument],
) {
    if let fastn_unresolved::UR::UnResolved(None) = v {
        *v = fastn_unresolved::UR::Resolved(());
        return;
    }

    let (argument, inner_v) = if let fastn_unresolved::UR::UnResolved(Some(inner_v)) = v {
        // see if any of the arguments are of type caption or body
        // assume there is only one such argument, because otherwise arguments would have failed
        // to resolve
        match arguments.iter().find(|v| {
            if is_caption {
                v.is_caption()
            } else {
                v.is_body()
            }
        }) {
            Some(a) => (a, inner_v),
            None => {
                *v = fastn_unresolved::UR::Invalid(fastn_section::Error::UnexpectedCaption);
                return;
            }
        }
    } else {
        return;
    };

    match crate::resolver::arguments::argument(argument, inner_v) {
        Ok(Some(_p)) => {
            todo!()
            // *v = fastn_unresolved::UR::Resolved(p)
        }
        Ok(None) => {
            todo!()
        }
        Err(e) => *v = fastn_unresolved::UR::Invalid(e),
    }
}

fn argument(
    _argument: &fastn_resolved::Argument,
    _value: &fastn_section::HeaderValue,
) -> Result<Option<fastn_resolved::Property>, fastn_section::Error> {
    todo!()
}
