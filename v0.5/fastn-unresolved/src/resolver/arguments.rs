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
) -> bool {
    let mut resolved = true;
    resolved &= caption_or_body(caption, true, arguments, properties);
    resolved &= caption_or_body(body, false, arguments, properties);
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
            Ok(Some(d)) => {
                *p = fastn_unresolved::UR::Resolved(d);
                resolved &= true;
            }
            Ok(None) => resolved = false,
            Err(e) => {
                *p = fastn_unresolved::UR::Invalid(e);
                resolved &= true;
            }
        }
    }

    resolved
    // TODO: check if any required argument is missing (should only be done when everything is
    //       resolved, how do we track this resolution?
    //       maybe this function can return a bool to say everything is resolved? but if everything
    //       is marked resolved, we have an issue, maybe we put something extra in properties in
    //       unresolved state, and resolve only when this is done?
}

fn caption_or_body(
    v: &mut fastn_unresolved::UR<Option<fastn_section::HeaderValue>, ()>,
    is_caption: bool,
    arguments: &[fastn_resolved::Argument],
    properties: &mut Vec<
        fastn_unresolved::UR<fastn_unresolved::Property, fastn_resolved::Property>,
    >,
) -> bool {
    if let fastn_unresolved::UR::UnResolved(None) = v {
        *v = fastn_unresolved::UR::Resolved(());
        return true;
    }

    let inner_v = if let fastn_unresolved::UR::UnResolved(Some(inner_v)) = v {
        // see if any of the arguments are of type caption or body
        // assume there is only one such argument, because otherwise arguments would have failed
        // to resolve
        inner_v
    } else {
        return true;
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
            true
        }
        Ok(None) => false,
        Err(e) => {
            *v = fastn_unresolved::UR::Invalid(e);
            true
        }
    }
}

enum Property<'a> {
    Field(&'a str),
    Caption,
    Body,
}

impl Property<'_> {
    fn source(&self) -> fastn_resolved::PropertySource {
        match self {
            Property::Field(f) => fastn_resolved::PropertySource::Header {
                name: f.to_string(),
                mutable: false,
            },
            Property::Body => fastn_resolved::PropertySource::Body,
            Property::Caption => fastn_resolved::PropertySource::Caption,
        }
    }
}

fn resolve_argument(
    arguments: &[fastn_resolved::Argument],
    property: Property,
    value: &fastn_section::HeaderValue,
) -> Result<Option<fastn_resolved::Property>, fastn_section::Error> {
    let argument = match arguments.iter().find(|v| match property {
        Property::Caption => v.is_caption(),
        Property::Body => v.is_body(),
        Property::Field(ref f) => &v.name == f,
    }) {
        Some(a) => a,
        None => return Err(fastn_section::Error::UnexpectedCaption), // TODO: do better
    };

    match argument.kind.kind {
        fastn_resolved::Kind::String => resolve_string(&property, value),
        _ => todo!(),
    }
}

fn resolve_string(
    property: &Property,
    value: &fastn_section::HeaderValue,
) -> Result<Option<fastn_resolved::Property>, fastn_section::Error> {
    if let Some(v) = value.as_plain_string() {
        return Ok(Some(fastn_resolved::Property {
            value: fastn_resolved::PropertyValue::Value {
                value: fastn_resolved::Value::String {
                    text: v.to_string(),
                },
                is_mutable: false,
                line_number: 0,
            },
            source: property.source(),
            condition: None,
            line_number: 0,
        }));
    };

    todo!()
}
