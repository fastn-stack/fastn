#[allow(clippy::too_many_arguments)]
pub fn arguments(
    arguments: &[fastn_resolved::Argument],
    caption: &mut fastn_unresolved::UR<Option<fastn_section::HeaderValue>, ()>,
    properties: &mut Vec<
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
    caption_or_body(caption, true, arguments, properties);
    caption_or_body(body, false, arguments, properties);
    for p in properties.iter_mut() {
        let inner_p = if let fastn_unresolved::UR::UnResolved(inner_p) = p {
            inner_p
        } else {
            continue;
        };
        match resolve_argument(
            arguments,
            Property::Field(inner_p.name.str()),
            &inner_p.value,
        ) {
            Ok(Some(d)) => *p = fastn_unresolved::UR::Resolved(d),
            Ok(None) => {}
            Err(e) => *p = fastn_unresolved::UR::Invalid(e),
        }
    }
}

fn caption_or_body(
    v: &mut fastn_unresolved::UR<Option<fastn_section::HeaderValue>, ()>,
    is_caption: bool,
    arguments: &[fastn_resolved::Argument],
    properties: &mut Vec<
        fastn_unresolved::UR<fastn_unresolved::Property, fastn_resolved::Property>,
    >,
) {
    if let fastn_unresolved::UR::UnResolved(None) = v {
        *v = fastn_unresolved::UR::Resolved(());
        return;
    }

    let inner_v = if let fastn_unresolved::UR::UnResolved(Some(inner_v)) = v {
        // see if any of the arguments are of type caption or body
        // assume there is only one such argument, because otherwise arguments would have failed
        // to resolve
        inner_v
    } else {
        return;
    };

    match resolve_argument(
        arguments,
        if is_caption {
            Property::Caption
        } else {
            Property::Body
        },
        inner_v,
    ) {
        Ok(Some(p)) => {
            *v = fastn_unresolved::UR::Resolved(());
            properties.push(fastn_unresolved::UR::Resolved(p));
        }
        Ok(None) => {}
        Err(e) => *v = fastn_unresolved::UR::Invalid(e),
    }
}

enum Property<'a> {
    Field(&'a str),
    Caption,
    Body,
}

fn resolve_argument(
    arguments: &[fastn_resolved::Argument],
    property: Property,
    _value: &fastn_section::HeaderValue,
) -> Result<Option<fastn_resolved::Property>, fastn_section::Error> {
    let _argument = match arguments.iter().find(|v| {
        match property {
            Property::Caption => v.is_caption(),
            Property::Body => v.is_body(),
            Property::Field(ref f) => &v.name == f,
        }
        // if is_caption {
        //     v.is_caption()
        // } else {
        //     v.is_body()
        // }
    }) {
        Some(a) => a,
        None => return Err(fastn_section::Error::UnexpectedCaption), // TODO: do better
    };

    todo!()
}
