#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentDefinition {
    pub name: String,
    pub arguments: Vec<Argument>,
    pub definition: Component,
}

pub const COMPONENT: &str = "component";

impl ComponentDefinition {
    fn new(name: &str, arguments: Vec<Argument>, definition: Component) -> ComponentDefinition {
        ComponentDefinition {
            name: name.to_string(),
            arguments,
            definition,
        }
    }

    pub fn is_component_definition(section: &ftd::p11::Section) -> bool {
        section.kind.as_ref().map_or(false, |s| s.eq(COMPONENT))
    }

    pub fn from_p1(
        section: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::ast::Result<ComponentDefinition> {
        if !Self::is_component_definition(section) {
            return ftd::ast::parse_error(
                format!(
                    "Section is not component definition section, found `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }

        if section.sub_sections.len() != 1 {
            return ftd::ast::parse_error(
                format!(
                    "Component definition should be exactly one, found `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }

        let arguments = {
            let mut arguments = vec![];
            for header in section.headers.0.iter() {
                arguments.push(Argument::from_p1_header(header, doc_id)?);
            }
            arguments
        };

        let definition = Component::from_p1(section.sub_sections.first().unwrap(), doc_id)?;

        Ok(ComponentDefinition::new(
            section.name.as_str(),
            arguments,
            definition,
        ))
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Component {
    pub name: String,
    pub properties: Vec<Property>,
    pub iteration: Option<Loop>,
    pub children: Vec<Component>,
}

impl Component {
    fn new(
        name: &str,
        properties: Vec<Property>,
        iteration: Option<Loop>,
        children: Vec<Component>,
    ) -> Component {
        Component {
            name: name.to_string(),
            properties,
            iteration,
            children,
        }
    }

    pub(crate) fn is_component(section: &ftd::p11::Section) -> bool {
        section.kind.is_none() && !section.name.starts_with(ftd::ast::utils::REFERENCE)
    }

    pub(crate) fn from_p1(
        section: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::ast::Result<Component> {
        if !Self::is_component(section) {
            return ftd::ast::parse_error(
                format!("Section is not ComponentDefinition, found `{:?}`", section),
                doc_id,
                section.line_number,
            );
        }

        let properties = {
            let mut properties = vec![];
            for header in section.headers.0.iter() {
                let name = header.get_key();
                if name.eq(ftd::ast::utils::LOOP) {
                    continue;
                }
                properties.push(Property::from_p1_header(
                    header,
                    doc_id,
                    PropertySource::Header {
                        mutable: ftd::ast::utils::is_variable_mutable(name.as_str()),
                        name,
                    },
                )?);
            }
            if let Some(ref caption) = section.caption {
                properties.push(Property::from_p1_header(
                    caption,
                    doc_id,
                    PropertySource::Caption,
                )?);
            }

            if let Some(ftd::p11::Body { ref value, .. }) = section.body {
                properties.push(Property::from_value(
                    Some(value.to_owned()),
                    PropertySource::Body,
                ));
            }
            properties
        };

        let children = {
            let mut children = vec![];
            for subsection in section.sub_sections.iter() {
                children.push(Component::from_p1(subsection, doc_id)?);
            }
            children
        };

        let iteration = Loop::from_headers(&section.headers, doc_id)?;

        Ok(Component::new(
            section.name.as_str(),
            properties,
            iteration,
            children,
        ))
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Argument {
    pub name: String,
    pub kind: ftd::ast::VariableKind,
    pub mutable: bool,
    pub value: Option<ftd::ast::VariableValue>,
}

impl Argument {
    fn is_argument(header: &ftd::p11::Header) -> bool {
        header.get_kind().is_some()
    }

    fn new(
        name: &str,
        kind: ftd::ast::VariableKind,
        mutable: bool,
        value: Option<ftd::ast::VariableValue>,
    ) -> Argument {
        Argument {
            name: name.to_string(),
            kind,
            mutable,
            value,
        }
    }

    fn from_p1_header(header: &ftd::p11::Header, doc_id: &str) -> ftd::ast::Result<Argument> {
        if !Self::is_argument(header) {
            return ftd::ast::parse_error(
                format!("Header is not argument, found `{:?}`", header),
                doc_id,
                header.get_line_number(),
            );
        }

        let kind = ftd::ast::VariableKind::get_kind(
            header.get_kind().as_ref().unwrap().as_str(),
            doc_id,
            header.get_line_number(),
        )?;

        let value =
            ftd::ast::VariableValue::from_header_with_modifier(header, doc_id, &kind.modifier)?
                .inner();

        let name = header.get_key();

        Ok(Argument::new(
            name.as_str(),
            kind,
            ftd::ast::utils::is_variable_mutable(name.as_str()),
            value,
        ))
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Property {
    pub value: Option<ftd::ast::VariableValue>,
    pub source: PropertySource,
}

impl Property {
    fn is_property(header: &ftd::p11::Header) -> bool {
        header.get_kind().is_none()
    }

    fn new(value: Option<ftd::ast::VariableValue>, source: PropertySource) -> Property {
        Property { value, source }
    }

    fn from_p1_header(
        header: &ftd::p11::Header,
        doc_id: &str,
        source: PropertySource,
    ) -> ftd::ast::Result<Property> {
        if !Self::is_property(header) || header.get_key().eq(ftd::ast::utils::LOOP) {
            return ftd::ast::parse_error(
                format!("Header is not property, found `{:?}`", header),
                doc_id,
                header.get_line_number(),
            );
        }

        let value = ftd::ast::VariableValue::from_p1_header(header).inner();

        Ok(Property::new(value, source))
    }

    fn from_value(value: Option<String>, source: PropertySource) -> Property {
        let value = ftd::ast::VariableValue::from_value(&value).inner();
        Property::new(value, source)
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PropertySource {
    Caption,
    Body,
    Header { name: String, mutable: bool },
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Loop {
    on: String,
    alias: String,
}

impl Loop {
    fn new(on: &str, alias: &str) -> Loop {
        Loop {
            on: on.to_string(),
            alias: alias.to_string(),
        }
    }

    fn from_headers(headers: &ftd::p11::Headers, doc_id: &str) -> ftd::ast::Result<Option<Loop>> {
        let loop_header = headers
            .0
            .iter()
            .find(|v| v.get_key().eq(ftd::ast::utils::LOOP));
        let loop_header = if let Some(loop_header) = loop_header {
            loop_header
        } else {
            return Ok(None);
        };

        let loop_statement = loop_header
            .get_value(doc_id)?
            .ok_or(ftd::ast::Error::ParseError {
                message: "Loop statement is blank".to_string(),
                doc_id: doc_id.to_string(),
                line_number: loop_header.get_line_number(),
            })?;

        let (on, alias) = ftd::ast::utils::split_at(loop_statement.as_str(), ftd::ast::utils::AS);

        if !on.starts_with(ftd::ast::utils::REFERENCE) {
            return ftd::ast::parse_error(
                format!(
                    "Loop should be on some reference, found: `{}`. Help: use `${}` instead",
                    on, on
                ),
                doc_id,
                loop_header.get_line_number(),
            );
        }

        let alias = {
            if let Some(alias) = alias {
                if !alias.starts_with(ftd::ast::utils::REFERENCE) {
                    return ftd::ast::parse_error(
                        format!(
                            "Loop alias should start with reference, found: `{}`. Help: use `${}` instead",
                            alias, alias
                        ),
                        doc_id,
                        loop_header.get_line_number(),
                    );
                }
                alias
                    .trim_start_matches(ftd::ast::utils::REFERENCE)
                    .to_string()
            } else {
                "object".to_string()
            }
        };

        Ok(Some(Loop::new(
            on.trim_start_matches(ftd::ast::utils::REFERENCE),
            alias.as_str(),
        )))
    }
}
