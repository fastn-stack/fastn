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
    pub children: Vec<Component>,
}

impl Component {
    fn new(name: &str, properties: Vec<Property>, children: Vec<Component>) -> Component {
        Component {
            name: name.to_string(),
            properties,
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
                properties.push(Property::from_p1_header(header, doc_id)?);
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

        Ok(Component::new(section.name.as_str(), properties, children))
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
    pub name: String,
    pub mutable: bool,
    pub value: Option<ftd::ast::VariableValue>,
}

impl Property {
    fn is_property(header: &ftd::p11::Header) -> bool {
        header.get_kind().is_none()
    }

    fn new(name: &str, mutable: bool, value: Option<ftd::ast::VariableValue>) -> Property {
        Property {
            name: name.to_string(),
            mutable,
            value,
        }
    }

    fn from_p1_header(header: &ftd::p11::Header, doc_id: &str) -> ftd::ast::Result<Property> {
        if !Self::is_property(header) {
            return ftd::ast::parse_error(
                format!("Header is not property, found `{:?}`", header),
                doc_id,
                header.get_line_number(),
            );
        }

        let value = ftd::ast::VariableValue::from_p1_header(header, doc_id).inner();

        let name = header.get_key();

        Ok(Property::new(
            name.as_str(),
            ftd::ast::utils::is_variable_mutable(name.as_str()),
            value,
        ))
    }
}
