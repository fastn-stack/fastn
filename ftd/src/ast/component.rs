use crate::ast::kind::{HeaderValue, HeaderValues};

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentDefinition {
    pub name: String,
    pub arguments: Vec<Argument>,
    pub definition: Component,
    pub css: Option<String>,
    pub line_number: usize,
}

pub const COMPONENT: &str = "component";

impl ComponentDefinition {
    fn new(
        name: &str,
        arguments: Vec<Argument>,
        definition: Component,
        css: Option<String>,
        line_number: usize,
    ) -> ComponentDefinition {
        ComponentDefinition {
            name: name.to_string(),
            arguments,
            definition,
            css,
            line_number,
        }
    }

    pub fn is_component_definition(section: &ftd_p1::Section) -> bool {
        section.kind.as_ref().map_or(false, |s| s.eq(COMPONENT))
    }

    pub fn from_p1(
        section: &ftd_p1::Section,
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

        let (css, arguments) =
            ftd::ast::utils::get_css_and_fields_from_headers(&section.headers, doc_id)?;

        let definition = Component::from_p1(section.sub_sections.first().unwrap(), doc_id)?;

        Ok(ComponentDefinition::new(
            section.name.as_str(),
            arguments,
            definition,
            css,
            section.line_number,
        ))
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Component {
    pub id: Option<String>,
    pub name: String,
    pub properties: Vec<Property>,
    pub iteration: Option<Loop>,
    pub condition: Option<ftd::ast::Condition>,
    pub events: Vec<Event>,
    pub children: Vec<Component>,
    #[serde(rename = "line-number")]
    pub line_number: usize,
}

impl Component {
    #[allow(clippy::too_many_arguments)]
    fn new(
        id: Option<String>,
        name: &str,
        properties: Vec<Property>,
        iteration: Option<Loop>,
        condition: Option<ftd::ast::Condition>,
        events: Vec<Event>,
        children: Vec<Component>,
        line_number: usize,
    ) -> Component {
        Component {
            id,
            name: name.to_string(),
            properties,
            iteration,
            condition,
            events,
            children,
            line_number,
        }
    }

    pub(crate) fn is_component(section: &ftd_p1::Section) -> bool {
        section.kind.is_none() && !section.name.starts_with(ftd::ast::utils::REFERENCE)
    }

    pub(crate) fn from_p1(section: &ftd_p1::Section, doc_id: &str) -> ftd::ast::Result<Component> {
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
                if name.eq(ftd::ast::utils::LOOP)
                    || name.eq(ftd::ast::utils::FOR)
                    || Event::get_event_name(name.as_str()).is_some()
                    || ftd::ast::utils::is_condition(header.get_key().as_str(), &header.get_kind())
                {
                    continue;
                }
                properties.push(Property::from_p1_header(
                    header,
                    doc_id,
                    PropertySource::Header {
                        mutable: ftd::ast::utils::is_variable_mutable(name.as_str()),
                        name: name
                            .trim_start_matches(ftd::ast::utils::REFERENCE)
                            .to_string(),
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

            if let Some(ftd_p1::Body {
                ref value,
                line_number,
            }) = section.body
            {
                properties.push(Property::from_value(
                    Some(value.to_owned()),
                    PropertySource::Body,
                    line_number,
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
        let events = Event::from_headers(&section.headers, doc_id)?;
        let condition = ftd::ast::Condition::from_headers(&section.headers, doc_id)?;
        let id = ftd::ast::utils::get_component_id(&section.headers, doc_id)?;

        Ok(Component::new(
            id,
            section.name.as_str(),
            properties,
            iteration,
            condition,
            events,
            children,
            section.line_number,
        ))
    }

    pub fn from_variable_value(
        key: &str,
        value: ftd::ast::VariableValue,
        doc_id: &str,
    ) -> ftd::ast::Result<Component> {
        match value {
            ftd::ast::VariableValue::Optional { value, .. } if value.is_some() => {
                Component::from_variable_value(key, value.unwrap(), doc_id)
            }
            ftd::ast::VariableValue::Optional { line_number, .. } => Ok(ftd::ast::Component {
                id: None,
                name: key.to_string(),
                properties: vec![],
                iteration: None,
                condition: None,
                events: vec![],
                children: vec![],
                line_number,
            }),
            ftd::ast::VariableValue::Constant { line_number, .. } => Ok(ftd::ast::Component {
                id: None,
                name: key.to_string(),
                properties: vec![],
                iteration: None,
                condition: None,
                events: vec![],
                children: vec![],
                line_number,
            }),
            ftd::ast::VariableValue::List {
                value,
                line_number,
                condition,
            } => {
                let mut children = vec![];
                for val in value {
                    children.push(Component::from_variable_value(
                        val.key.as_str(),
                        val.value,
                        doc_id,
                    )?);
                }
                Ok(ftd::ast::Component {
                    id: None,
                    name: key.to_string(),
                    properties: vec![],
                    iteration: None,
                    condition,
                    events: vec![],
                    children,
                    line_number,
                })
            }
            ftd::ast::VariableValue::Record {
                name,
                caption,
                headers,
                body,
                line_number,
                values,
                condition,
            } => {
                let mut properties = vec![];
                if let Some(caption) = caption.as_ref() {
                    properties.push(ftd::ast::Property {
                        value: caption.to_owned(),
                        source: ftd::ast::PropertySource::Caption,
                        condition: caption.condition_expression(),
                        line_number,
                    });
                }
                for header in headers.0.iter() {
                    if header.key.eq(ftd::ast::utils::LOOP)
                        || header.key.eq(ftd::ast::utils::FOR)
                        || Event::get_event_name_from_header_value(header).is_some()
                        || ftd::ast::utils::is_condition(header.key.as_str(), &header.kind)
                    {
                        continue;
                    }
                    properties.push(ftd::ast::Property {
                        value: header.value.to_owned(),
                        source: ftd::ast::PropertySource::Header {
                            name: header.key.to_string(),
                            mutable: header.mutable,
                        },
                        condition: header.condition.to_owned(),
                        line_number,
                    });
                }
                if let Some(body) = body {
                    properties.push(Property::from_value(
                        Some(body.value),
                        PropertySource::Body,
                        body.line_number,
                    ));
                }

                let iteration = Loop::from_ast_headers(&headers, doc_id)?;
                let events = Event::from_ast_headers(&headers, doc_id)?;

                let mut children = vec![];

                for child in values {
                    children.push(Component::from_variable_value(
                        child.key.as_str(),
                        child.value,
                        doc_id,
                    )?);
                }

                Ok(ftd::ast::Component {
                    id: None,
                    name,
                    properties,
                    iteration,
                    condition,
                    events,
                    children,
                    line_number,
                })
            }
            ftd::ast::VariableValue::String {
                value,
                line_number,
                source: value_source,
                condition,
            } => Ok(ftd::ast::Component {
                id: None,
                name: key.to_string(),
                properties: vec![Property::from_value(
                    Some(value),
                    value_source.to_property_source(),
                    line_number,
                )],
                iteration: None,
                condition,
                events: vec![],
                children: vec![],
                line_number,
            }),
        }
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

pub type Argument = ftd::ast::Field;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Property {
    pub value: ftd::ast::VariableValue,
    pub source: PropertySource,
    pub condition: Option<String>,
    #[serde(rename = "line-number")]
    pub line_number: usize,
}

impl Property {
    fn is_property(header: &ftd_p1::Header) -> bool {
        header.get_kind().is_none()
    }

    fn new(
        value: ftd::ast::VariableValue,
        source: PropertySource,
        condition: Option<String>,
        line_number: usize,
    ) -> Property {
        Property {
            value,
            source,
            condition,
            line_number,
        }
    }

    fn from_p1_header(
        header: &ftd_p1::Header,
        doc_id: &str,
        source: PropertySource,
    ) -> ftd::ast::Result<Property> {
        if !Self::is_property(header)
            || header.get_key().eq(ftd::ast::utils::LOOP)
            || header.get_key().eq(ftd::ast::utils::FOR)
            || Event::get_event_name(header.get_key().as_str()).is_some()
        {
            return ftd::ast::parse_error(
                format!("Header is not property, found `{:?}`", header),
                doc_id,
                header.get_line_number(),
            );
        }

        let value = ftd::ast::VariableValue::from_p1_header(header, doc_id)?;

        Ok(Property::new(
            value,
            source,
            header.get_condition(),
            header.get_line_number(),
        ))
    }

    fn from_value(value: Option<String>, source: PropertySource, line_number: usize) -> Property {
        let value =
            ftd::ast::VariableValue::from_value(&value, source.to_value_source(), line_number);
        Property::new(value, source, None, line_number)
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PropertySource {
    #[default]
    Caption,
    Body,
    #[serde(rename = "header")]
    Header {
        name: String,
        mutable: bool,
    },
}

impl PropertySource {
    pub fn is_equal(&self, other: &PropertySource) -> bool {
        match self {
            PropertySource::Caption | PropertySource::Body => self.eq(other),
            PropertySource::Header { name, .. } => matches!(other, PropertySource::Header {
                    name: other_name, ..
               } if other_name.eq(name)),
        }
    }

    pub fn to_value_source(&self) -> ftd::ast::ValueSource {
        match self {
            ftd::ast::PropertySource::Caption => ftd::ast::ValueSource::Caption,
            ftd::ast::PropertySource::Body => ftd::ast::ValueSource::Body,
            ftd::ast::PropertySource::Header { name, mutable } => ftd::ast::ValueSource::Header {
                name: name.to_owned(),
                mutable: mutable.to_owned(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Loop {
    pub on: String,
    pub alias: String,
    pub loop_counter_alias: Option<String>,
    #[serde(rename = "line-number")]
    pub line_number: usize,
}

impl Loop {
    fn new(on: &str, alias: &str, loop_counter_alias: Option<String>, line_number: usize) -> Loop {
        Loop {
            on: on.to_string(),
            alias: alias.to_string(),
            loop_counter_alias,
            line_number,
        }
    }

    fn get_loop_parameters(
        loop_statement: &str,
        is_for_loop: bool,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::ast::Result<(String, String, Option<String>)> {
        if is_for_loop {
            let (pair, on) = ftd::ast::utils::split_at(loop_statement, ftd::ast::utils::IN);

            let on = on.ok_or(ftd::ast::Error::Parse {
                message: "Statement \"for\" needs a list to operate on".to_string(),
                doc_id: doc_id.to_string(),
                line_number,
            })?;

            let (alias, loop_counter_alias) = ftd::ast::utils::split_at(pair.as_str(), ", ");

            Ok((alias, on, loop_counter_alias))
        } else {
            use colored::Colorize;

            println!(
                "{}",
                "Warning: \"$loop$\" is deprecated, use \"for\" instead".bright_yellow()
            );

            let (on, alias) = ftd::ast::utils::split_at(loop_statement, ftd::ast::utils::AS);

            let alias = if let Some(alias) = alias {
                if !alias.starts_with(ftd::ast::utils::REFERENCE) {
                    return ftd::ast::parse_error(
                    format!(
                        "Loop alias should start with reference, found: `{}`. Help: use `${}` instead",
                        alias, alias
                    ),
                    doc_id,
                    line_number,
                );
                }

                alias
            } else {
                "object".to_string()
            };

            Ok((alias, on, None))
        }
    }

    fn from_ast_headers(headers: &HeaderValues, doc_id: &str) -> ftd::ast::Result<Option<Loop>> {
        let loop_header = headers
            .0
            .iter()
            .find(|v| [ftd::ast::utils::LOOP, ftd::ast::utils::FOR].contains(&v.key.as_str()));
        let loop_header = if let Some(loop_header) = loop_header {
            loop_header
        } else {
            return Ok(None);
        };

        let loop_statement = loop_header.value.string(doc_id)?;

        let is_for_loop = loop_header.key.eq(ftd::ast::utils::FOR);

        let (alias, on, loop_counter_alias) = Self::get_loop_parameters(
            loop_statement.as_str(),
            is_for_loop,
            doc_id,
            loop_header.line_number,
        )?;

        if !on.starts_with(ftd::ast::utils::REFERENCE) && !on.starts_with(ftd::ast::utils::CLONE) {
            return ftd::ast::parse_error(
                format!(
                    "Loop should be on some reference, found: `{}`. Help: use `${}` instead",
                    on, on
                ),
                doc_id,
                loop_header.line_number,
            );
        }

        let alias = alias
            .trim_start_matches(ftd::ast::utils::REFERENCE)
            .to_string();

        Ok(Some(Loop::new(
            on.as_str(),
            alias.as_str(),
            loop_counter_alias,
            loop_header.line_number,
        )))
    }

    fn from_headers(headers: &ftd_p1::Headers, doc_id: &str) -> ftd::ast::Result<Option<Loop>> {
        let loop_header = headers.0.iter().find(|v| {
            [ftd::ast::utils::LOOP, ftd::ast::utils::FOR].contains(&v.get_key().as_str())
        });
        let loop_header = if let Some(loop_header) = loop_header {
            loop_header
        } else {
            return Ok(None);
        };

        let loop_statement = loop_header
            .get_value(doc_id)?
            .ok_or(ftd::ast::Error::Parse {
                message: "Loop statement is blank".to_string(),
                doc_id: doc_id.to_string(),
                line_number: loop_header.get_line_number(),
            })?;

        let is_for_loop = loop_header.get_key().eq(ftd::ast::utils::FOR);

        let (alias, on, loop_counter_alias) = Self::get_loop_parameters(
            loop_statement.as_str(),
            is_for_loop,
            doc_id,
            loop_header.get_line_number(),
        )?;

        if !on.starts_with(ftd::ast::utils::REFERENCE) && !on.starts_with(ftd::ast::utils::CLONE) {
            return ftd::ast::parse_error(
                format!(
                    "Loop should be on some reference, found: `{}`. Help: use `${}` instead",
                    on, on
                ),
                doc_id,
                loop_header.get_line_number(),
            );
        }

        let alias = alias
            .trim_start_matches(ftd::ast::utils::REFERENCE)
            .to_string();

        Ok(Some(Loop::new(
            on.as_str(),
            alias.as_str(),
            loop_counter_alias,
            loop_header.get_line_number(),
        )))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Event {
    pub name: String,
    pub action: String,
    #[serde(rename = "line-number")]
    pub line_number: usize,
}

impl Event {
    fn new(name: &str, action: &str, line_number: usize) -> Event {
        Event {
            name: name.to_string(),
            action: action.to_string(),
            line_number,
        }
    }

    fn get_event_name_from_header_value(header_value: &HeaderValue) -> Option<String> {
        let mut name = header_value.key.clone();
        if header_value.mutable {
            name = format!("${}", name);
        }
        Event::get_event_name(name.as_str())
    }

    fn get_event_name(input: &str) -> Option<String> {
        if !(input.starts_with("$on-") && input.ends_with(ftd::ast::utils::REFERENCE)) {
            return None;
        }
        Some(
            input
                .trim_start_matches("$on-")
                .trim_end_matches(ftd::ast::utils::REFERENCE)
                .to_string(),
        )
    }

    fn from_ast_headers(headers: &HeaderValues, doc_id: &str) -> ftd::ast::Result<Vec<Event>> {
        let mut events = vec![];
        for header in headers.0.iter() {
            if let Some(event) = Event::from_ast_header(header, doc_id)? {
                events.push(event);
            }
        }
        Ok(events)
    }

    fn from_ast_header(header: &HeaderValue, doc_id: &str) -> ftd::ast::Result<Option<Event>> {
        let event_name = if let Some(name) = Event::get_event_name_from_header_value(header) {
            name
        } else {
            return Ok(None);
        };

        let action = header.value.string(doc_id)?;

        Ok(Some(Event::new(
            event_name.as_str(),
            action.as_str(),
            header.line_number,
        )))
    }

    fn from_headers(headers: &ftd_p1::Headers, doc_id: &str) -> ftd::ast::Result<Vec<Event>> {
        let mut events = vec![];
        for header in headers.0.iter() {
            if let Some(event) = Event::from_header(header, doc_id)? {
                events.push(event);
            }
        }
        Ok(events)
    }

    fn from_header(header: &ftd_p1::Header, doc_id: &str) -> ftd::ast::Result<Option<Event>> {
        let event_name = if let Some(name) = Event::get_event_name(header.get_key().as_str()) {
            name
        } else {
            return Ok(None);
        };

        let action = header.get_value(doc_id)?.ok_or(ftd::ast::Error::Parse {
            message: "Event cannot be empty".to_string(),
            doc_id: doc_id.to_string(),
            line_number: header.get_line_number(),
        })?;

        Ok(Some(Event::new(
            event_name.as_str(),
            action.as_str(),
            header.get_line_number(),
        )))
    }
}
