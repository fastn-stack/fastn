#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentDefinition {
    pub name: String,
    pub arguments: Vec<Argument>,
    pub definition: fastn_type::Component,
    pub css: Option<fastn_type::PropertyValue>,
    pub line_number: usize,
}

impl ComponentDefinition {
    pub(crate) fn new(
        name: &str,
        arguments: Vec<Argument>,
        definition: Component,
        css: Option<fastn_type::PropertyValue>,
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

    pub(crate) fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        use itertools::Itertools;

        let component_definition = ast.get_component_definition(doc.name)?;
        let arguments = component_definition
            .arguments
            .iter()
            .map(|v| v.name.to_string())
            .collect_vec();

        let definition_name_with_arguments =
            (component_definition.name.as_str(), arguments.as_slice());

        Component::scan_ast_component(
            component_definition.definition,
            Some(definition_name_with_arguments),
            doc,
        )?;

        Argument::scan_ast_fields(component_definition.arguments, doc, &Default::default())?;

        Ok(())
    }

    pub(crate) fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ComponentDefinition>> {
        use ftd::interpreter::PropertyValueExt;

        let component_definition = ast.get_component_definition(doc.name)?;
        let name = doc.resolve_name(component_definition.name.as_str());

        let css = if let Some(ref css) = component_definition.css {
            Some(try_ok_state!(fastn_type::PropertyValue::from_ast_value(
                ftd_ast::VariableValue::String {
                    value: css.to_string(),
                    line_number: component_definition.line_number(),
                    source: ftd_ast::ValueSource::Default,
                    condition: None
                },
                doc,
                false,
                Some(&fastn_type::Kind::string().into_kind_data()),
            )?))
        } else {
            None
        };

        let mut arguments = try_ok_state!(Argument::from_ast_fields(
            component_definition.name.as_str(),
            component_definition.arguments,
            doc,
            &Default::default(),
        )?);

        let definition_name_with_arguments =
            (component_definition.name.as_str(), arguments.as_mut_slice());
        let definition = try_ok_state!(Component::from_ast_component(
            component_definition.definition,
            &mut Some(definition_name_with_arguments),
            doc,
        )?);
        if let Some(iteration) = definition.iteration.as_ref() {
            return Err(ftd::interpreter::Error::ParseError {
                message: "The component definition cannot have loop. Help: use container component as it's parent"
                    .to_string(),
                doc_id: doc.name.to_string(),
                line_number: iteration.line_number,
            });
        }
        Ok(ftd::interpreter::StateWithThing::new_thing(
            ComponentDefinition::new(
                name.as_str(),
                arguments,
                definition,
                css,
                component_definition.line_number,
            ),
        ))
    }

    pub fn to_value(&self, kind: &fastn_type::KindData) -> fastn_type::Value {
        fastn_type::Value::UI {
            name: self.name.to_string(),
            kind: kind.to_owned(),
            component: self.definition.to_owned(),
        }
    }
}

pub type Argument = ftd::interpreter::Field;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Component {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub properties: Vec<Property>,
    pub iteration: Box<Option<Loop>>,
    pub condition: Box<Option<ftd::interpreter::Expression>>,
    pub events: Vec<Event>,
    pub children: Vec<Component>,
    pub source: ComponentSource,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum ComponentSource {
    #[default]
    Declaration,
    Variable,
}

impl Component {
    pub(crate) fn from_name(name: &str) -> Component {
        Component {
            id: None,
            name: name.to_string(),
            properties: vec![],
            iteration: Box::new(None),
            condition: Box::new(None),
            events: vec![],
            children: vec![],
            source: Default::default(),
            line_number: 0,
        }
    }

    pub fn get_interpreter_value_of_argument(
        &self,
        argument_name: &str,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<Option<fastn_type::Value>> {
        let component_definition = doc.get_component(self.name.as_str(), 0).unwrap();
        let argument = component_definition
            .arguments
            .iter()
            .find(|v| v.name.eq(argument_name))
            .unwrap();
        argument.get_default_interpreter_value(doc, self.properties.as_slice())
    }

    pub fn get_interpreter_property_value_of_all_arguments(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::Map<fastn_type::PropertyValue>> {
        let component_definition = doc.get_component(self.name.as_str(), 0).unwrap();
        let mut property_values: ftd::Map<fastn_type::PropertyValue> = Default::default();
        for argument in component_definition.arguments.iter() {
            if let Some(property_value) =
                argument.get_default_interpreter_property_value(self.properties.as_slice())?
            {
                property_values.insert(argument.name.to_string(), property_value);
            }
        }
        Ok(property_values)
    }

    // Todo: Remove this function after removing 0.3
    pub fn get_children_property(&self) -> Option<fastn_type::Property> {
        self.get_children_properties().first().map(|v| v.to_owned())
    }

    pub fn get_children_properties(&self) -> Vec<fastn_type::Property> {
        ftd::interpreter::utils::get_children_properties_from_properties(&self.properties)
    }

    pub fn get_children(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<Vec<Component>> {
        use ftd::interpreter::PropertyValueExt;

        let property = if let Some(property) = self.get_children_property() {
            property
        } else {
            return Ok(vec![]);
        };

        let value = property.value.clone().resolve(doc, property.line_number)?;
        if let fastn_type::Value::UI { component, .. } = value {
            return Ok(vec![component]);
        }
        if let fastn_type::Value::List { data, kind } = value {
            if kind.is_ui() {
                let mut children = vec![];
                for value in data {
                    let value = value.resolve(doc, property.line_number)?;
                    if let fastn_type::Value::UI { component, .. } = value {
                        children.push(component);
                    }
                }
                return Ok(children);
            }
        }

        Ok(vec![])
    }

    pub fn get_kwargs(
        &self,
        doc: &ftd::interpreter::Document,
        kwargs_name: &str,
    ) -> ftd::interpreter::Result<ftd::Map<String>> {
        use ftd::interpreter::ValueExt;
        use ftd::js::fastn_type_functions::PropertyValueExt;

        let property = match self.get_interpreter_value_of_argument(kwargs_name, &doc.tdoc())? {
            Some(property) => property,
            None => {
                return Err(ftd::interpreter::Error::OtherError(format!(
                    "kw-args '{}' does not exists on component.",
                    kwargs_name
                )));
            }
        };

        let kwargs = property
            .kwargs(doc.name.as_str(), self.line_number)?
            .iter()
            .map(|(name, value)| {
                let value = match value.to_value().get_string_data() {
                    Some(v) => v,
                    None => {
                        return Err(ftd::interpreter::Error::ParseError {
                            message: "Could not parse keyword argument value as string."
                                .to_string(),
                            doc_id: doc.name.clone(),
                            line_number: value.line_number(),
                        });
                    }
                };

                Ok((name.to_string(), value))
            })
            .collect::<Result<ftd::Map<String>, _>>()?;

        Ok(kwargs)
    }

    pub(crate) fn is_loop(&self) -> bool {
        self.iteration.is_some()
    }

    pub(crate) fn is_variable(&self) -> bool {
        self.source.eq(&fastn_type::ComponentSource::Variable)
    }

    pub(crate) fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        let component_invocation = ast.get_component_invocation(doc.name)?;
        Component::scan_ast_component(component_invocation, None, doc)
    }

    pub(crate) fn scan_ast_component(
        ast_component: ftd_ast::ComponentInvocation,
        definition_name_with_arguments: Option<(&str, &[String])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        Property::scan_ast_children(ast_component.children, definition_name_with_arguments, doc)?;
        match definition_name_with_arguments {
            Some((definition, _))
                if ast_component.name.eq(definition)
                    || ast_component
                        .name
                        .starts_with(format!("{definition}.").as_str()) => {}
            _ => doc.scan_thing(ast_component.name.as_str(), ast_component.line_number)?,
        }

        let mut loop_object_name_and_kind = None;
        if let Some(v) = ast_component.iteration {
            loop_object_name_and_kind = Some(doc.resolve_name(v.alias.as_str()));
            Loop::scan_ast_loop(v, definition_name_with_arguments, doc)?;
        };

        if let Some(v) = ast_component.condition {
            ftd::interpreter::Expression::scan_ast_condition(
                v,
                definition_name_with_arguments,
                &loop_object_name_and_kind,
                doc,
            )?;
        }

        Event::scan_ast_events(
            ast_component.events,
            definition_name_with_arguments,
            &loop_object_name_and_kind,
            doc,
        )?;

        Property::scan_ast_properties(
            ast_component.properties,
            definition_name_with_arguments,
            &loop_object_name_and_kind,
            doc,
        )?;

        Ok(())
    }

    pub(crate) fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Component>> {
        let component_invocation = ast.get_component_invocation(doc.name)?;
        Component::from_ast_component(component_invocation, &mut None, doc)
    }

    pub(crate) fn from_ast_component(
        ast_component: ftd_ast::ComponentInvocation,
        definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Component>> {
        let name = doc.resolve_name(ast_component.name.as_str());

        // If the component is from `module` type argument
        ftd::interpreter::utils::insert_module_thing(
            &fastn_type::Kind::ui().into_kind_data(),
            ast_component.name.as_str(),
            name.as_str(),
            definition_name_with_arguments,
            ast_component.line_number(),
            doc,
        )
        .ok();

        let mut loop_object_name_and_kind = None;
        let iteration = if let Some(v) = ast_component.iteration {
            let iteration =
                try_ok_state!(Loop::from_ast_loop(v, definition_name_with_arguments, doc)?);
            loop_object_name_and_kind = Some((
                iteration.alias.to_string(),
                iteration.loop_object_as_argument(doc)?,
                iteration.loop_counter_alias.to_owned(),
            ));
            Some(iteration)
        } else {
            None
        };

        let condition = if let Some(v) = ast_component.condition {
            Some(try_ok_state!(
                ftd::interpreter::Expression::from_ast_condition(
                    v,
                    definition_name_with_arguments,
                    &loop_object_name_and_kind,
                    doc,
                )?
            ))
        } else {
            None
        };

        let events = try_ok_state!(Event::from_ast_events(
            ast_component.events,
            definition_name_with_arguments,
            &loop_object_name_and_kind,
            doc,
        )?);

        if let Some(component) = try_ok_state!(Component::variable_component_from_ast(
            ast_component.name.as_str(),
            definition_name_with_arguments,
            doc,
            &iteration,
            &condition,
            &loop_object_name_and_kind,
            events.as_slice(),
            &ast_component.properties,
            &ast_component.children,
            ast_component.line_number
        )?) {
            return Ok(ftd::interpreter::StateWithThing::new_thing(component));
        }

        let properties = try_ok_state!(Property::from_ast_properties_and_children(
            ast_component.properties,
            ast_component.children,
            ast_component.name.as_str(),
            definition_name_with_arguments,
            &loop_object_name_and_kind,
            doc,
            ast_component.line_number,
        )?);
        if let Some((_name, arguments)) = definition_name_with_arguments {
            Self::assert_no_private_properties_while_invocation(&properties, arguments)?;
        } else if let ftd::interpreter::Thing::Component(c) =
            doc.get_thing(name.as_str(), ast_component.line_number)?
        {
            Self::assert_no_private_properties_while_invocation(&properties, &c.arguments)?;
        }

        let id = ast_component.id;

        Ok(ftd::interpreter::StateWithThing::new_thing(Component {
            id,
            name,
            properties,
            iteration: Box::new(iteration),
            condition: Box::new(condition),
            events,
            children: vec![],
            source: Default::default(),
            line_number: ast_component.line_number,
        }))
    }

    pub fn assert_no_private_properties_while_invocation(
        properties: &[Property],
        arguments: &[Argument],
    ) -> ftd::interpreter::Result<()> {
        let mut private_arguments: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for arg in arguments.iter() {
            if !arg.access_modifier.is_public() {
                private_arguments.insert(arg.name.clone());
            }
        }

        for property in properties.iter() {
            if let PropertySource::Header { name, .. } = &property.source {
                if private_arguments.contains(name.as_str()) {
                    return Err(ftd::interpreter::Error::InvalidAccessError {
                        message: format!(
                            "{} argument is private and can't be accessed on \
                        invocation",
                            name
                        ),
                        line_number: property.line_number,
                    });
                }
            }
        }

        Ok(())
    }

    /// Component which is a variable
    /// -- s:
    /// where `s` is a variable of `ftd.ui` type
    #[allow(clippy::too_many_arguments)]
    fn variable_component_from_ast(
        name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
        doc: &mut ftd::interpreter::TDoc,
        iteration: &Option<Loop>,
        condition: &Option<ftd::interpreter::Expression>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        events: &[Event],
        ast_properties: &Vec<ftd_ast::Property>,
        ast_children: &Vec<ftd_ast::ComponentInvocation>,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Option<Component>>> {
        use ftd::interpreter::{PropertyValueExt, PropertyValueSourceExt};

        let name = doc.resolve_name(name);

        if definition_name_with_arguments.is_none()
            || doc
                .resolve_name(definition_name_with_arguments.as_ref().unwrap().0)
                .ne(&name)
        {
            let mut var_name = if let Some(value) =
                ftd::interpreter::utils::get_argument_for_reference_and_remaining(
                    name.as_str(),
                    doc,
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                    line_number,
                )? {
                Some((
                    value.2.get_reference_name(name.as_str(), doc),
                    Some(value.0),
                ))
            } else {
                None
            };

            if var_name.is_none() {
                if let Ok(variable) = doc.search_variable(name.as_str(), line_number) {
                    try_ok_state!(variable);
                    var_name = Some((name.to_string(), None));
                }
            }

            if let Some((name, arg)) = var_name {
                let mut properties = vec![];
                if let Some(arg) = arg {
                    if arg.kind.is_module() {
                        let component_name = {
                            let (m_name, _) = match arg
                                .value
                                .as_ref()
                                .unwrap()
                                .clone()
                                .resolve(doc, line_number)?
                            {
                                fastn_type::Value::Module { name, things } => (name, things),
                                t => {
                                    return ftd::interpreter::utils::e2(
                                        format!("Expected module, found: {:?}", t),
                                        doc.name,
                                        line_number,
                                    );
                                }
                            };
                            let component_name = definition_name_with_arguments.as_ref().unwrap().0;
                            format!(
                                "{}#{}",
                                m_name,
                                name.trim_start_matches(
                                    format!("{}#{}.{}.", doc.name, component_name, arg.name)
                                        .as_str()
                                )
                            )
                        };

                        properties = try_ok_state!(Property::from_ast_properties_and_children(
                            ast_properties.to_owned(),
                            ast_children.to_owned(),
                            component_name.as_str(),
                            definition_name_with_arguments,
                            loop_object_name_and_kind,
                            doc,
                            line_number,
                        )?);
                    }
                }

                return Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                    Component {
                        id: None,
                        name,
                        properties,
                        iteration: Box::new(iteration.to_owned()),
                        condition: Box::new(condition.to_owned()),
                        events: events.to_vec(),
                        children: vec![],
                        source: fastn_type::ComponentSource::Variable,
                        line_number,
                    },
                )));
            }
        }

        Ok(ftd::interpreter::StateWithThing::new_thing(None))
    }
}

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum PropertySource {
    #[default]
    Caption,
    Body,
    Header {
        name: String,
        mutable: bool,
    },
    Subsection,
    Default,
}

impl PropertySource {
    pub fn is_equal(&self, other: &PropertySource) -> bool {
        match self {
            PropertySource::Caption
            | PropertySource::Body
            | PropertySource::Subsection
            | PropertySource::Default => self.eq(other),
            PropertySource::Header { name, .. } => matches!(other, PropertySource::Header {
                    name: other_name, ..
               } if other_name.eq(name)),
        }
    }

    pub fn is_default(&self) -> bool {
        matches!(self, PropertySource::Default)
    }

    pub fn header(name: &str) -> PropertySource {
        PropertySource::Header {
            name: name.to_string(),
            mutable: false,
        }
    }
}

impl From<ftd_ast::PropertySource> for PropertySource {
    fn from(item: ftd_ast::PropertySource) -> Self {
        match item {
            ftd_ast::PropertySource::Caption => PropertySource::Caption,
            ftd_ast::PropertySource::Body => PropertySource::Body,
            ftd_ast::PropertySource::Header { name, mutable } => {
                PropertySource::Header { name, mutable }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Property {
    pub value: fastn_type::PropertyValue,
    pub source: fastn_type::PropertySource,
    pub condition: Option<ftd::interpreter::Expression>,
    pub line_number: usize,
}

impl Property {
    fn from_ast_properties_and_children(
        ast_properties: Vec<ftd_ast::Property>,
        ast_children: Vec<ftd_ast::ComponentInvocation>,
        component_name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<Property>>> {
        let mut properties = try_ok_state!(Property::from_ast_properties(
            ast_properties,
            component_name,
            definition_name_with_arguments,
            loop_object_name_and_kind,
            doc,
            line_number,
        )?);

        // todo: validate_duplicate_properties() a property cannot be repeat if it's not list

        validate_children_kind_property_against_children(
            properties.as_slice(),
            ast_children.as_slice(),
            doc.name,
        )?;

        if let Some(property) = try_ok_state!(Property::from_ast_children(
            ast_children,
            component_name,
            definition_name_with_arguments,
            doc,
        )?) {
            properties.push(property);
        }

        return Ok(ftd::interpreter::StateWithThing::new_thing(properties));

        fn validate_children_kind_property_against_children(
            properties: &[Property],
            ast_children: &[ftd_ast::ComponentInvocation],
            doc_id: &str,
        ) -> ftd::interpreter::Result<()> {
            use itertools::Itertools;

            let properties = properties
                .iter()
                .filter(|v| v.value.kind().inner_list().is_subsection_ui())
                .collect_vec();

            if properties.is_empty() {
                return Ok(());
            }

            let first_property = properties.first().unwrap();

            if properties.len() > 1 {
                return ftd::interpreter::utils::e2(
                    "Can't pass multiple children",
                    doc_id,
                    first_property.line_number,
                );
            }

            if !ast_children.is_empty() {
                return ftd::interpreter::utils::e2(
                    "Can't have children passed in both subsection and header",
                    doc_id,
                    first_property.line_number,
                );
            }

            if first_property.condition.is_some() {
                return ftd::interpreter::utils::e2(
                    "Not supporting condition for children",
                    doc_id,
                    first_property.line_number,
                );
            }

            Ok(())
        }
    }

    fn get_argument_for_children(component_arguments: &[Argument]) -> Option<&Argument> {
        component_arguments
            .iter()
            .find(|v| v.kind.kind.clone().inner_list().is_subsection_ui())
    }

    fn from_ast_children(
        ast_children: Vec<ftd_ast::ComponentInvocation>,
        component_name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Option<Property>>> {
        if ast_children.is_empty() {
            return Ok(ftd::interpreter::StateWithThing::new_thing(None));
        }

        let line_number = ast_children.first().unwrap().line_number;
        let component_arguments = try_ok_state!(Argument::for_component(
            component_name,
            definition_name_with_arguments,
            doc,
            line_number,
        )?);

        let _argument = Property::get_argument_for_children(&component_arguments).ok_or(
            ftd::interpreter::Error::ParseError {
                message: "SubSection is unexpected".to_string(),
                doc_id: doc.name.to_string(),
                line_number,
            },
        )?;

        let children = {
            let mut children = vec![];
            for child in ast_children {
                children.push(try_ok_state!(Component::from_ast_component(
                    child,
                    definition_name_with_arguments,
                    doc
                )?));
            }
            children
        };

        let value = fastn_type::PropertyValue::Value {
            value: fastn_type::Value::List {
                data: children
                    .into_iter()
                    .map(|v| fastn_type::PropertyValue::Value {
                        line_number: v.line_number,
                        value: fastn_type::Value::UI {
                            name: v.name.to_string(),
                            kind: fastn_type::Kind::subsection_ui().into_kind_data(),
                            component: v,
                        },
                        is_mutable: false,
                    })
                    .collect(),
                kind: fastn_type::Kind::subsection_ui().into_kind_data(),
            },
            is_mutable: false,
            line_number,
        };

        Ok(ftd::interpreter::StateWithThing::new_thing(Some(
            Property {
                value,
                source: fastn_type::PropertySource::Subsection,
                condition: None,
                line_number,
            },
        )))
    }

    fn scan_ast_children(
        ast_children: Vec<ftd_ast::ComponentInvocation>,
        definition_name_with_arguments: Option<(&str, &[String])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        if ast_children.is_empty() {
            return Ok(());
        }

        for child in ast_children {
            Component::scan_ast_component(child, definition_name_with_arguments, doc)?;
        }

        Ok(())
    }

    fn scan_ast_properties(
        ast_properties: Vec<ftd_ast::Property>,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        for property in ast_properties {
            Property::scan_ast_property(
                property,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            )?;
        }
        Ok(())
    }

    fn scan_ast_property(
        ast_property: ftd_ast::Property,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        use ftd::interpreter::PropertyValueExt;

        fastn_type::PropertyValue::scan_ast_value_with_argument(
            ast_property.value.to_owned(),
            doc,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )?;

        if let Some(ref v) = ast_property.condition {
            ftd::interpreter::Expression::scan_ast_condition(
                ftd_ast::Condition::new(v, ast_property.line_number),
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            )?;
        }

        Ok(())
    }

    fn from_ast_properties(
        ast_properties: Vec<ftd_ast::Property>,
        component_name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<Property>>> {
        let mut properties = vec![];
        let component_arguments = try_ok_state!(Argument::for_component_or_web_component(
            component_name,
            definition_name_with_arguments,
            doc,
            line_number,
        )?);

        let kw_args = component_arguments.iter().find(|a| a.kind.is_kwargs());

        let mut extra_arguments = vec![];

        for property in ast_properties {
            match Property::from_ast_property(
                property.clone(),
                component_name,
                component_arguments.as_slice(),
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            ) {
                Ok(property) => {
                    properties.push(try_ok_state!(property));
                }
                Err(e) => {
                    if kw_args.is_some() {
                        if let Some((name, value)) =
                            get_extra_argument_property_value(property, doc.name.to_string())?
                        {
                            extra_arguments.push((name, value));
                            continue;
                        };
                    }

                    return Err(e);
                }
            };
        }

        if let Some(kw_args) = kw_args {
            properties.push(fastn_type::Property {
                value: fastn_type::PropertyValue::Value {
                    value: fastn_type::Value::KwArgs {
                        arguments: std::collections::BTreeMap::from_iter(extra_arguments),
                    },
                    is_mutable: false,
                    line_number: kw_args.line_number,
                },
                source: fastn_type::PropertySource::Header {
                    name: kw_args.name.clone(),
                    mutable: false,
                },
                condition: None,
                line_number: kw_args.line_number,
            });
        }

        try_ok_state!(search_things_for_module(
            component_name,
            properties.as_slice(),
            doc,
            component_arguments.as_slice(),
            definition_name_with_arguments,
            line_number,
        )?);

        check_if_property_is_provided_for_required_argument(
            &component_arguments,
            &properties,
            component_name,
            line_number,
            doc.name,
        )?;

        Ok(ftd::interpreter::StateWithThing::new_thing(properties))
    }

    fn from_ast_property(
        ast_property: ftd_ast::Property,
        component_name: &str,
        component_arguments: &[Argument],
        definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Property>> {
        use ftd::interpreter::PropertyValueExt;

        let argument = try_ok_state!(Property::get_argument_for_property(
            &ast_property,
            component_name,
            component_arguments,
            doc,
        )?);

        let value = try_ok_state!(fastn_type::PropertyValue::from_ast_value_with_argument(
            ast_property.value.to_owned(),
            doc,
            argument.mutable,
            Some(&argument.kind),
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )?);

        let condition = if let Some(ref v) = ast_property.condition {
            Some(try_ok_state!(
                ftd::interpreter::Expression::from_ast_condition(
                    ftd_ast::Condition::new(v, ast_property.line_number),
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                    doc,
                )?
            ))
        } else {
            None
        };

        if ast_property.value.is_null() && !argument.kind.is_optional() {
            return ftd::interpreter::utils::e2(
                format!(
                    "Excepted Value for argument {} in component {}",
                    argument.name, component_name
                ),
                doc.name,
                ast_property.line_number,
            );
        }

        let source = {
            let mut source = ast_property.source.into();
            if let fastn_type::PropertySource::Header { name, .. } = &mut source {
                *name = argument.name;
            }
            source
        };

        Ok(ftd::interpreter::StateWithThing::new_thing(Property {
            value,
            source,
            condition,
            line_number: ast_property.line_number,
        }))
    }

    fn get_argument_for_property(
        ast_property: &ftd_ast::Property,
        component_name: &str,
        component_argument: &[Argument],
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Argument>> {
        match &ast_property.source {
            ftd_ast::PropertySource::Caption => Ok(ftd::interpreter::StateWithThing::new_thing(
                component_argument
                    .iter()
                    .find(|v| v.is_caption())
                    .ok_or(ftd::interpreter::Error::ParseError {
                        message: format!(
                            "Caption type argument not found for component `{}`",
                            component_name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number: ast_property.line_number,
                    })
                    .map(ToOwned::to_owned)?,
            )),
            ftd_ast::PropertySource::Body => Ok(ftd::interpreter::StateWithThing::new_thing(
                component_argument
                    .iter()
                    .find(|v| v.is_body())
                    .ok_or(ftd::interpreter::Error::ParseError {
                        message: format!(
                            "Body type argument not found for component `{}`",
                            component_name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number: ast_property.line_number,
                    })
                    .map(ToOwned::to_owned)?,
            )),
            ftd_ast::PropertySource::Header { name, mutable } => {
                let (name, remaining) = ftd::interpreter::utils::split_at(name, ".");
                let mut argument = component_argument
                    .iter()
                    .find(|v| v.name.eq(name.as_str()))
                    .ok_or(ftd::interpreter::Error::ParseError {
                        message: format!(
                            "Header type `{}` mutable: `{}` argument not found for component `{}`",
                            name, mutable, component_name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number: ast_property.line_number,
                    })?
                    .to_owned();
                if !argument.mutable.eq(mutable) {
                    let mutable = if argument.mutable {
                        "mutable"
                    } else {
                        "immutable"
                    };
                    return ftd::interpreter::utils::e2(
                        format!("Expected `{}` for {}", mutable, argument.name),
                        doc.name,
                        ast_property.line_number,
                    );
                }

                if let Some(variant) = remaining {
                    try_ok_state!(argument.update_with_or_type_variant(
                        doc,
                        variant.as_str(),
                        ast_property.line_number
                    )?);
                }

                Ok(ftd::interpreter::StateWithThing::new_thing(argument))
            }
        }
    }

    pub(crate) fn get_local_argument(&self, component_name: &str) -> Option<String> {
        if let Some(reference) = self.value.get_reference_or_clone() {
            if let Some(reference) = reference.strip_prefix(format!("{}.", component_name).as_str())
            {
                return Some(reference.to_string());
            }
        }
        None
    }
}

fn get_extra_argument_property_value(
    property: ftd_ast::Property,
    doc_id: String,
) -> ftd::interpreter::Result<Option<(String, fastn_type::PropertyValue)>> {
    if let ftd_ast::PropertySource::Header { name, .. } = property.source.clone() {
        let line_number = property.value.line_number();
        let value = match property.value {
            ftd_ast::VariableValue::String { value, .. } => value,
            value => {
                return Err(ftd::interpreter::Error::InvalidKind {
                    doc_id,
                    line_number: value.line_number(),
                    message: "kw-args currently support only string values.".to_string(),
                })
            }
        };

        return Ok(Some((
            name,
            fastn_type::PropertyValue::Value {
                value: fastn_type::Value::new_string(&value),
                is_mutable: false,
                line_number,
            },
        )));
    }

    Ok(None)
}

fn check_if_property_is_provided_for_required_argument(
    component_arguments: &[ftd::interpreter::Field],
    properties: &[fastn_type::Property],
    component_name: &str,
    line_number: usize,
    doc_id: &str,
) -> ftd::interpreter::Result<()> {
    for argument in component_arguments {
        if !argument.is_value_required() || argument.kind.is_kwargs() {
            continue;
        }
        if argument
            .get_default_interpreter_property_value(properties)
            .map(|v| v.is_none())
            .unwrap_or(true)
        {
            return Err(ftd::interpreter::Error::ParseError {
                message: format!(
                    "Property `{}` of component `{}` is not passed",
                    argument.name, component_name
                ),
                doc_id: doc_id.to_string(),
                line_number,
            });
        }
    }
    Ok(())
}

fn search_things_for_module(
    component_name: &str,
    properties: &[fastn_type::Property],
    doc: &mut ftd::interpreter::TDoc,
    arguments: &[ftd::interpreter::Argument],
    definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
    line_number: usize,
) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<()>> {
    for argument in arguments.iter() {
        if !argument.kind.is_module() {
            continue;
        }
        let sources = argument.to_sources();
        let property = ftd::interpreter::utils::find_properties_by_source(
            sources.as_slice(),
            properties,
            doc.name,
            argument,
            argument.line_number,
        )?;
        if property.len() != 1 {
            return ftd::interpreter::utils::e2(
                format!(
                    "Expected one value for `module` type argument `{}`, found `{}` values",
                    argument.name,
                    property.len()
                ),
                doc.name,
                line_number,
            );
        }
        let module_property = property.first().unwrap();
        // TODO: Remove unwrap()

        let (m_name, things) = get_module_name_and_thing(
            module_property,
            doc,
            definition_name_with_arguments,
            argument,
        )?;

        let mut m_alias;
        {
            let current_parsed_document = if let Some(state) = {
                match &mut doc.bag {
                    ftd::interpreter::tdoc::BagOrState::Bag(_) => None,
                    ftd::interpreter::tdoc::BagOrState::State(s) => Some(s),
                }
            } {
                state.parsed_libs.get_mut(state.id.as_str()).unwrap()
            } else {
                return doc.err("not found", m_name, "search_thing", line_number);
            };
            let (module, alias) = ftd_ast::utils::get_import_alias(m_name.as_str());
            if !current_parsed_document
                .doc_aliases
                .contains_key(alias.as_str())
            {
                current_parsed_document
                    .doc_aliases
                    .insert(alias.to_string(), module.to_string());
            }
            m_alias = alias;
        }

        if let Some(m) = doc.aliases.get(m_alias.as_str()) {
            m_alias = m.to_string();
        }

        let mut unresolved_thing = None;

        for (thing, _expected_kind) in things {
            let mut new_doc_name = doc.name.to_string();
            let mut new_doc_aliases = doc.aliases.clone();

            // If the module name (value) is coming from the argument of the component then we
            // need to change doc to the new-doc, else if it's coming from property then no need
            // to change the doc.
            if module_property.source.is_default() {
                // This is needed because the component can be exported from some other module
                // so, we need to fetch this module name in module_name
                // -- import: foo
                // export: bar
                //
                // So the bar component is actually present in foo module and we need foo as
                // value of module_name.
                let component_name = doc
                    .get_thing(component_name, line_number)
                    .map(|v| v.name())
                    .unwrap_or(component_name.to_string());
                let module_name =
                    ftd::interpreter::utils::get_doc_name(component_name.as_str(), doc.name);

                if let Some(state) = doc.state() {
                    let parsed_document = state.parsed_libs.get(module_name.as_str()).unwrap();
                    new_doc_name = parsed_document.name.to_string();
                    new_doc_aliases = parsed_document.doc_aliases.clone();
                }
            }

            let mut new_doc = match &mut doc.bag {
                ftd::interpreter::BagOrState::Bag(bag) => {
                    ftd::interpreter::TDoc::new(&new_doc_name, &new_doc_aliases, bag)
                }
                ftd::interpreter::BagOrState::State(state) => {
                    ftd::interpreter::TDoc::new_state(&new_doc_name, &new_doc_aliases, state)
                }
            };

            let mut m_alias = m_alias.clone();
            if let Some(m) = new_doc.aliases.get(m_alias.as_str()) {
                m_alias = m.to_string();
            }

            let thing_real_name = format!("{}#{}", m_alias, thing);

            if unresolved_thing.is_some() {
                new_doc.scan_thing(&thing_real_name, line_number)?;
            } else {
                let result = new_doc.search_thing(&thing_real_name, line_number)?;
                if !result.is_thing() {
                    unresolved_thing = Some(result);
                } else {
                    //Todo: check with kind, if kind matches with expected_kind
                    try_ok_state!(result);
                }
            }
        }

        if let Some(unresolved_thing) = unresolved_thing {
            try_ok_state!(unresolved_thing);
        }
    }
    Ok(ftd::interpreter::StateWithThing::new_thing(()))
}

fn get_module_name_and_thing(
    module_property: &fastn_type::Property,
    doc: &mut ftd::interpreter::TDoc,
    definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
    component_argument: &ftd::interpreter::Argument,
) -> ftd::interpreter::Result<(String, ftd::Map<ftd::interpreter::ModuleThing>)> {
    use ftd::interpreter::{PropertyExt, PropertyValueExt};

    let default_things = {
        let value = if let Some(ref value) = component_argument.value {
            value.clone().resolve(doc, module_property.line_number)?
        } else {
            return ftd::interpreter::utils::e2(
                "Cannot find component argument value for module",
                doc.name,
                component_argument.line_number,
            );
        };

        if let Some(thing) = value.module_thing_optional() {
            thing.clone()
        } else {
            return ftd::interpreter::utils::e2(
                "Cannot find component argument value for module",
                doc.name,
                component_argument.line_number,
            );
        }
    };
    if let Some(module_name) = module_property.value.get_reference_or_clone() {
        if let Some((argument, ..)) =
            ftd::interpreter::utils::get_component_argument_for_reference_and_remaining(
                module_name,
                doc.name,
                definition_name_with_arguments,
                module_property.line_number,
            )?
        {
            if let Some(ref mut property_value) = argument.value {
                if let fastn_type::PropertyValue::Value { value, .. } = property_value {
                    if let Some((name, thing)) = value.mut_module_optional() {
                        thing.extend(default_things);
                        return Ok((name.to_string(), thing.clone()));
                    } else {
                        return ftd::interpreter::utils::e2(
                            format!("Expected module, found: {:?}", property_value),
                            doc.name,
                            module_property.line_number,
                        );
                    }
                }
                match property_value
                    .clone()
                    .resolve(doc, module_property.line_number)?
                {
                    fastn_type::Value::Module { name, things } => return Ok((name, things)),
                    t => {
                        return ftd::interpreter::utils::e2(
                            format!("Expected module, found: {:?}", t),
                            doc.name,
                            module_property.line_number,
                        )
                    }
                }
            }
        }
    }

    match module_property
        .resolve(doc, &Default::default())?
        // TODO: Remove unwrap()
        .unwrap()
    {
        fastn_type::Value::Module { name, things } => Ok((name, things)),
        t => ftd::interpreter::utils::e2(
            format!("Expected module, found: {:?}", t),
            doc.name,
            module_property.line_number,
        ),
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Loop {
    pub on: fastn_type::PropertyValue,
    pub alias: String,
    pub loop_counter_alias: Option<String>,
    pub line_number: usize,
}

impl Loop {
    fn new(
        on: fastn_type::PropertyValue,
        alias: &str,
        loop_counter_alias: Option<String>,
        line_number: usize,
    ) -> Loop {
        Loop {
            on,
            alias: alias.to_string(),
            line_number,
            loop_counter_alias,
        }
    }

    pub(crate) fn loop_object_as_argument(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::Argument> {
        let kind = self.loop_object_kind(doc.name)?;
        Ok(ftd::interpreter::Argument {
            name: self.alias.to_string(),
            kind: fastn_type::KindData::new(kind),
            mutable: self.on.is_mutable(),
            value: Some(self.on.to_owned()),
            line_number: self.on.line_number(),
            access_modifier: Default::default(),
        })
    }

    pub(crate) fn loop_object_kind(
        &self,
        doc_id: &str,
    ) -> ftd::interpreter::Result<fastn_type::Kind> {
        let kind = self.on.kind();
        match kind {
            fastn_type::Kind::List { kind } => Ok(kind.as_ref().to_owned()),
            t => ftd::interpreter::utils::e2(
                format!("Expected list kind, found: {:?}", t),
                doc_id,
                self.line_number,
            ),
        }
    }

    fn scan_ast_loop(
        ast_loop: ftd_ast::Loop,
        definition_name_with_arguments: Option<(&str, &[String])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        use ftd::interpreter::PropertyValueExt;

        fastn_type::PropertyValue::scan_string_with_argument(
            ast_loop.on.as_str(),
            doc,
            ast_loop.line_number,
            definition_name_with_arguments,
            &None,
        )?;

        Ok(())
    }

    fn from_ast_loop(
        ast_loop: ftd_ast::Loop,
        definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Loop>> {
        use ftd::interpreter::PropertyValueExt;

        let mut on = try_ok_state!(fastn_type::PropertyValue::from_string_with_argument(
            ast_loop.on.as_str(),
            doc,
            None,
            false,
            ast_loop.line_number,
            definition_name_with_arguments,
            &None,
        )?);

        if let Some(reference) = ast_loop.on.strip_prefix(ftd::interpreter::utils::REFERENCE) {
            if let Ok(ftd::interpreter::StateWithThing::Thing(t)) = doc.get_kind_with_argument(
                reference,
                ast_loop.line_number,
                definition_name_with_arguments,
                &None,
            ) {
                on.set_mutable(t.2);
            }
        }

        if ast_loop.on.starts_with(ftd::interpreter::utils::CLONE) {
            on.set_mutable(true);
        }

        Ok(ftd::interpreter::StateWithThing::new_thing(Loop::new(
            on,
            doc.resolve_name(ast_loop.alias.as_str()).as_str(),
            ast_loop
                .loop_counter_alias
                .map(|loop_counter_alias| doc.resolve_name(loop_counter_alias.as_str())),
            ast_loop.line_number,
        )))
    }

    pub fn children(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<(Vec<fastn_type::PropertyValue>, fastn_type::KindData)> {
        use ftd::interpreter::PropertyValueExt;

        let value = self.on.clone().resolve(doc, self.line_number)?;
        if let fastn_type::Value::List { data, kind } = value {
            Ok((data, kind))
        } else {
            ftd::interpreter::utils::e2(
                format!("Expected list type data, found: {:?}", self.on),
                doc.name,
                self.line_number,
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Event {
    pub name: fastn_type::EventName,
    pub action: fastn_type::FunctionCall,
    pub line_number: usize,
}

impl Event {
    fn from_ast_event(
        ast_event: ftd_ast::Event,
        definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Event>> {
        use ftd::interpreter::FunctionCallExt;

        let action = try_ok_state!(fastn_type::FunctionCall::from_string(
            ast_event.action.as_str(),
            doc,
            false,
            definition_name_with_arguments,
            loop_object_name_and_kind,
            ast_event.line_number,
        )?);

        if action.module_name.is_some() {
            let (function_name, _) = ftd::interpreter::utils::get_function_name_and_properties(
                ast_event.action.as_str(),
                doc.name,
                ast_event.line_number,
            )?;

            let reference = function_name.as_str().trim_start_matches('$');
            let reference_full_name = action.name.as_str();

            ftd::interpreter::utils::insert_module_thing(
                &action.kind,
                reference,
                reference_full_name,
                definition_name_with_arguments,
                ast_event.line_number,
                doc,
            )?;
        }

        let event_name = fastn_type::EventName::from_string(
            ast_event.name.as_str(),
            doc.name,
            ast_event.line_number,
        )?;

        Ok(ftd::interpreter::StateWithThing::new_thing(Event {
            name: event_name,
            action,
            line_number: ast_event.line_number,
        }))
    }

    fn from_ast_events(
        ast_events: Vec<ftd_ast::Event>,
        definition_name_with_arguments: &mut Option<(&str, &mut [Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<Event>>> {
        let mut events = vec![];
        for event in ast_events {
            events.push(try_ok_state!(Event::from_ast_event(
                event,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            )?));
        }
        Ok(ftd::interpreter::StateWithThing::new_thing(events))
    }

    fn scan_ast_events(
        ast_events: Vec<ftd_ast::Event>,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        for event in ast_events {
            Event::scan_ast_event(
                event,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            )?;
        }
        Ok(())
    }

    fn scan_ast_event(
        ast_event: ftd_ast::Event,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        use ftd::interpreter::FunctionCallExt;

        fastn_type::FunctionCall::scan_string(
            ast_event.action.as_str(),
            doc,
            definition_name_with_arguments,
            loop_object_name_and_kind,
            ast_event.line_number,
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum EventName {
    Click,
    MouseEnter,
    MouseLeave,
    ClickOutside,
    GlobalKey(Vec<String>),
    GlobalKeySeq(Vec<String>),
    Input,
    Change,
    Blur,
    Focus,
    RivePlay(String),
    RiveStateChange(String),
    RivePause(String),
}

impl EventName {
    pub(crate) fn from_string(
        e: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::EventName> {
        use itertools::Itertools;

        match e {
            "click" => Ok(EventName::Click),
            "mouse-enter" => Ok(EventName::MouseEnter),
            "mouse-leave" => Ok(EventName::MouseLeave),
            "click-outside" => Ok(EventName::ClickOutside),
            "input" => Ok(EventName::Input),
            "change" => Ok(EventName::Change),
            "blur" => Ok(EventName::Blur),
            "focus" => Ok(EventName::Focus),
            t if t.starts_with("global-key[") && t.ends_with(']') => {
                let keys = t
                    .trim_start_matches("global-key[")
                    .trim_end_matches(']')
                    .split('-')
                    .map(|v| v.to_string())
                    .collect_vec();
                Ok(EventName::GlobalKey(keys))
            }
            t if t.starts_with("global-key-seq[") && t.ends_with(']') => {
                let keys = t
                    .trim_start_matches("global-key-seq[")
                    .trim_end_matches(']')
                    .split('-')
                    .map(|v| v.to_string())
                    .collect_vec();
                Ok(EventName::GlobalKeySeq(keys))
            }
            t if t.starts_with("rive-play[") && t.ends_with(']') => {
                let timeline = t
                    .trim_start_matches("rive-play[")
                    .trim_end_matches(']')
                    .to_string();
                Ok(EventName::RivePlay(timeline))
            }
            t if t.starts_with("rive-state-change[") && t.ends_with(']') => {
                let state = t
                    .trim_start_matches("rive-state-change[")
                    .trim_end_matches(']')
                    .to_string();
                Ok(EventName::RiveStateChange(state))
            }
            t if t.starts_with("rive-pause[") && t.ends_with(']') => {
                let pause = t
                    .trim_start_matches("rive-pause[")
                    .trim_end_matches(']')
                    .to_string();
                Ok(EventName::RivePause(pause))
            }
            t => {
                ftd::interpreter::utils::e2(format!("`{}` event not found", t), doc_id, line_number)
            }
        }
    }

    pub(crate) fn is_click(&self) -> bool {
        self.eq(&EventName::Click)
    }
}
