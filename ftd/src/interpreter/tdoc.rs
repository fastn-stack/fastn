use fastn_resolved::{ComponentDefinition, Definition, Function, Record};
use ftd::interpreter::expression::ExpressionExt;
use ftd::interpreter::things::component::ComponentDefinitionExt;
use ftd::interpreter::things::or_type::OrTypeVariantExt;
use ftd::interpreter::things::record::RecordExt;
use ftd::interpreter::{FunctionExt, ThingExt};

#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: BagOrState<'a>,
}

#[derive(Debug, PartialEq)]
pub enum BagOrState<'a> {
    Bag(&'a indexmap::IndexMap<String, ftd::interpreter::Thing>),
    State(&'a mut ftd::interpreter::InterpreterState),
}

impl<'a> TDoc<'a> {
    pub fn new(
        name: &'a str,
        aliases: &'a ftd::Map<String>,
        bag: &'a indexmap::IndexMap<String, ftd::interpreter::Thing>,
    ) -> TDoc<'a> {
        TDoc {
            name,
            aliases,
            bag: BagOrState::Bag(bag),
        }
    }

    pub fn new_state(
        name: &'a str,
        aliases: &'a ftd::Map<String>,
        state: &'a mut ftd::interpreter::InterpreterState,
    ) -> TDoc<'a> {
        TDoc {
            name,
            aliases,
            bag: BagOrState::State(state),
        }
    }

    pub fn state(&'a self) -> Option<&'a &'a mut ftd::interpreter::InterpreterState> {
        match &self.bag {
            BagOrState::Bag(_) => None,
            BagOrState::State(s) => Some(s),
        }
    }

    pub fn resolve_module_name(&self, name: &str) -> String {
        ftd::interpreter::utils::resolve_module_name(name, self.name, self.aliases)
    }

    pub fn resolve_name(&self, name: &str) -> String {
        ftd::interpreter::utils::resolve_name(name, self.name, self.aliases)
    }

    pub fn bag(&'a self) -> &'a indexmap::IndexMap<String, ftd::interpreter::Thing> {
        match &self.bag {
            BagOrState::Bag(b) => b,
            BagOrState::State(s) => &s.bag,
        }
    }

    pub fn get_record(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Record> {
        match self.get_thing(name, line_number)? {
            ftd::interpreter::Thing::Record(r) => Ok(r),
            t => self.err(
                format!("Expected Record, found: `{t:?}`").as_str(),
                name,
                "get_record",
                line_number,
            ),
        }
    }

    pub fn get_or_type(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::OrType> {
        match self.get_thing(name, line_number)? {
            ftd::interpreter::Thing::OrType(ot) => Ok(ot),
            t => self.err(
                format!("Expected OrType, found: `{t:?}`").as_str(),
                name,
                "get_or_type",
                line_number,
            ),
        }
    }

    pub fn search_record(
        &mut self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::Record>> {
        match self.search_thing(name, line_number)? {
            ftd::interpreter::StateWithThing::State(s) => {
                Ok(ftd::interpreter::StateWithThing::new_state(*s))
            }
            ftd::interpreter::StateWithThing::Continue => {
                Ok(ftd::interpreter::StateWithThing::new_continue())
            }
            ftd::interpreter::StateWithThing::Thing(ftd::interpreter::Thing::Record(r)) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(r.clone()))
            }
            ftd::interpreter::StateWithThing::Thing(t) => self.err(
                format!("Expected Record, found: `{t:?}`").as_str(),
                name,
                "search_record",
                line_number,
            ),
        }
    }

    pub fn get_variable(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Variable> {
        match self.get_thing(name, line_number)? {
            ftd::interpreter::Thing::Variable(r) => Ok(r),
            t => self.err(
                format!("Expected Variable, found: `{t:?}`").as_str(),
                name,
                "get_variable",
                line_number,
            ),
        }
    }

    pub fn get_value(
        &'a self,
        line_number: usize,
        name: &'a str,
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
        use ftd::interpreter::PropertyValueExt;
        // TODO: name can be a.b.c, and a and a.b are records with right fields
        match self.get_thing(name, line_number)? {
            ftd::interpreter::Thing::Variable(v) => v.value.resolve(self, line_number),
            v => self.err("not a variable", v, "get_value", line_number),
        }
    }

    pub fn search_variable(
        &mut self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::Variable>> {
        match self.search_thing(name, line_number)? {
            ftd::interpreter::StateWithThing::State(s) => {
                Ok(ftd::interpreter::StateWithThing::new_state(*s))
            }
            ftd::interpreter::StateWithThing::Continue => {
                Ok(ftd::interpreter::StateWithThing::new_continue())
            }
            ftd::interpreter::StateWithThing::Thing(ftd::interpreter::Thing::Variable(r)) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(r))
            }
            ftd::interpreter::StateWithThing::Thing(t) => self.err(
                format!("Expected Variable, found: `{t:?}`").as_str(),
                name,
                "search_variable",
                line_number,
            ),
        }
    }

    pub fn eq(&'a self, name1: &'a str, name2: &'a str) -> bool {
        let name1 = self.resolve_name(name1);
        let name2 = self.resolve_name(name2);
        name1.eq(&name2)
    }

    pub(crate) fn resolve_reference_name(
        &self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<String> {
        Ok(if let Some(l) = name.strip_prefix('$') {
            let d =
                ftd::interpreter::utils::get_doc_name_and_remaining(l, self.name, line_number).0;
            if ftd::interpreter::utils::get_special_variable().contains(&d.as_str()) {
                return Ok(format!("${l}"));
            }
            format!("${}", self.resolve_name(l))
        } else {
            name.to_string()
        })
    }
    pub(crate) fn resolve(
        &self,
        name: &str,
        kind: &fastn_resolved::KindData,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
        self.resolve_with_inherited(name, kind, line_number, &Default::default())
    }

    pub(crate) fn resolve_with_inherited(
        &self,
        name: &str,
        kind: &fastn_resolved::KindData,
        line_number: usize,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
        use ftd::interpreter::PropertyValueExt;

        let (value, _var_name, _var_line_number, remaining) = if let Ok(v) =
            self.get_initial_variable_with_inherited(name, line_number, inherited_variables)
        {
            let mut value = v.0.value;
            for conditional in v.0.conditional_value.iter() {
                if conditional.condition.eval(self)? {
                    value = conditional.value.clone();
                    break;
                }
            }
            (value, v.0.name, v.0.line_number, v.1)
        } else if let Ok(v) = self.get_component(name, line_number) {
            (
                fastn_resolved::PropertyValue::Value {
                    value: v.to_value(kind),
                    is_mutable: false,
                    line_number: v.line_number,
                },
                v.name,
                v.line_number,
                None,
            )
        } else {
            return ftd::interpreter::utils::e2(
                format!("Cannot find 111 {name} in get_thing"),
                self.name,
                line_number,
            );
        };
        let value = value.resolve_with_inherited(self, line_number, inherited_variables)?;
        if let Some(remaining) = remaining {
            return resolve_(
                remaining.as_str(),
                &value,
                line_number,
                self,
                inherited_variables,
            );
        }
        return Ok(value);

        fn resolve_(
            name: &str,
            value: &fastn_resolved::Value,
            line_number: usize,
            doc: &ftd::interpreter::TDoc,
            inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        ) -> ftd::interpreter::Result<fastn_resolved::Value> {
            let (p1, p2) = ftd::interpreter::utils::split_at(name, ".");
            match value {
                fastn_resolved::Value::Record {
                    name: rec_name,
                    fields,
                } => {
                    let field = fields
                        .get(p1.as_str())
                        .ok_or(ftd::interpreter::Error::ParseError {
                            message: format!("Can't find field `{p1}` in record `{rec_name}`"),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?
                        .clone()
                        .resolve_with_inherited(doc, line_number, inherited_variables)?;
                    if let Some(p2) = p2 {
                        return resolve_(
                            p2.as_str(),
                            &field,
                            line_number,
                            doc,
                            inherited_variables,
                        );
                    }
                    Ok(field)
                }
                fastn_resolved::Value::List { data, kind } => {
                    let p1 = p1.parse::<usize>()?;
                    let value = data
                        .get(p1)
                        .ok_or(ftd::interpreter::Error::ParseError {
                            message: format!("Can't find index `{p1}` in list of kind `{kind:?}`"),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?
                        .clone()
                        .resolve_with_inherited(doc, line_number, inherited_variables)?;
                    if let Some(p2) = p2 {
                        return resolve_(
                            p2.as_str(),
                            &value,
                            line_number,
                            doc,
                            inherited_variables,
                        );
                    }
                    Ok(value)
                }
                t => ftd::interpreter::utils::e2(
                    format!("Expected record found `{t:?}`").as_str(),
                    doc.name,
                    line_number,
                ),
            }
        }
    }

    pub fn set_value(
        &'a self,
        name: &'a str,
        value: fastn_resolved::PropertyValue,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Variable> {
        let (mut variable, mut remaining) = self.get_initial_variable(name, line_number)?;

        if !variable.mutable {
            return ftd::interpreter::utils::e2(
                format!(
                    "The variable declaration `{}` is not mutable in line number {}",
                    variable.name, variable.line_number
                )
                .as_str(),
                self.name,
                line_number,
            );
        }

        if let Some((var, rem)) =
            find_variable_reference(&variable.value, remaining.clone(), self, line_number)?
        {
            variable = var;
            remaining = rem;
        }

        set_value_(&mut variable, value, remaining, self, line_number)?;

        return Ok(variable.clone());

        fn find_variable_reference(
            value: &fastn_resolved::PropertyValue,
            name: Option<String>,
            doc: &ftd::interpreter::TDoc,
            line_number: usize,
        ) -> ftd::interpreter::Result<Option<(fastn_resolved::Variable, Option<String>)>> {
            use ftd::interpreter::PropertyValueExt;

            let mut variable = None;
            let mut remaining = name;
            let mut value = value.clone();
            while let Some(reference) = value.reference_name() {
                let (var, rem) = doc.get_initial_variable(reference, line_number)?;
                value = var.value.clone();
                variable = Some(var);
                remaining = if let Some(remaining) = remaining {
                    Some(rem.map(|v| format!("{v}.{remaining}")).unwrap_or(remaining))
                } else {
                    rem
                };
            }

            if let fastn_resolved::PropertyValue::Clone { .. } = value {
                return Ok(variable.map(|v| (v, remaining)));
            }

            if let Some(ref remaining) = remaining {
                let (p1, p2) = ftd::interpreter::utils::split_at(remaining, ".");
                let value = value.value(doc.name, line_number)?.inner().ok_or(
                    ftd::interpreter::Error::ParseError {
                        message: format!(
                            "Value expected found null, `{value:?}` in line number {line_number}"
                        ),
                        doc_id: doc.name.to_string(),
                        line_number,
                    },
                )?;

                match value {
                    fastn_resolved::Value::Record {
                        name: rec_name,
                        fields,
                    } => {
                        let field_value = fields
                            .get(p1.as_str())
                            .ok_or(ftd::interpreter::Error::ParseError {
                                message: format!(
                                    "Expected field {p1} in record `{rec_name}` in line number {line_number}"
                                ),
                                doc_id: doc.name.to_string(),
                                line_number,
                            })?
                            .to_owned();
                        if let Some(variable) =
                            find_variable_reference(&field_value, p2, doc, line_number)?
                        {
                            return Ok(Some(variable));
                        }
                    }
                    t => {
                        return ftd::interpreter::utils::e2(
                            format!("Expected record, found `{t:?}` in line number {line_number}")
                                .as_str(),
                            doc.name,
                            line_number,
                        );
                    }
                }
            }

            Ok(variable.map(|v| (v, remaining)))
        }

        fn set_value_(
            variable: &mut fastn_resolved::Variable,
            value: fastn_resolved::PropertyValue,
            remaining: Option<String>,
            doc: &ftd::interpreter::TDoc,
            line_number: usize,
        ) -> ftd::interpreter::Result<()> {
            change_value(&mut variable.value, value, remaining, doc, line_number)?;
            Ok(())
        }

        fn change_value(
            value: &mut fastn_resolved::PropertyValue,
            set: fastn_resolved::PropertyValue,
            remaining: Option<String>,
            doc: &ftd::interpreter::TDoc,
            line_number: usize,
        ) -> ftd::interpreter::Result<()> {
            if let Some(remaining) = remaining {
                let (p1, p2) = ftd::interpreter::utils::split_at(remaining.as_str(), ".");
                match value {
                    fastn_resolved::PropertyValue::Value { value, .. } => match value {
                        fastn_resolved::Value::Record { name, fields } => {
                            let field = fields.get_mut(p1.as_str()).ok_or(
                                ftd::interpreter::Error::ParseError {
                                    message: format!("Can't find field `{p1}` in record `{name}`"),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                },
                            )?;
                            change_value(field, set, p2, doc, line_number)?;
                        }
                        t => {
                            return ftd::interpreter::utils::e2(
                                format!("Expected record, found `{t:?}`").as_str(),
                                doc.name,
                                line_number,
                            );
                        }
                    },
                    fastn_resolved::PropertyValue::Reference {
                        name,
                        kind,
                        is_mutable,
                        ..
                    }
                    | fastn_resolved::PropertyValue::Clone {
                        name,
                        kind,
                        is_mutable,
                        ..
                    } => {
                        let resolved_value = doc.resolve(name, kind, line_number)?;
                        *value = fastn_resolved::PropertyValue::Value {
                            value: resolved_value,
                            line_number,
                            is_mutable: *is_mutable,
                        };
                        change_value(value, set, Some(remaining), doc, line_number)?;
                    }
                    fastn_resolved::PropertyValue::FunctionCall(fastn_resolved::FunctionCall {
                        name,
                        kind,
                        is_mutable,
                        values,
                        ..
                    }) => {
                        let function = doc.get_function(name, line_number)?;
                        let resolved_value = function
                            .resolve(kind, values, doc, line_number)?
                            .ok_or(ftd::interpreter::Error::ParseError {
                                message: format!(
                                    "Expected return value of type {kind:?} for function {name}"
                                ),
                                doc_id: doc.name.to_string(),
                                line_number,
                            })?;
                        *value = fastn_resolved::PropertyValue::Value {
                            value: resolved_value,
                            line_number,
                            is_mutable: *is_mutable,
                        };
                        change_value(value, set, Some(remaining), doc, line_number)?;
                    }
                }
            } else if value.kind().inner().eq(&set.kind()) || value.kind().eq(&set.kind()) {
                *value = set;
            } else {
                return ftd::interpreter::utils::e2(
                    format!(
                        "Expected kind `{:?}`, found: \
                    `{:?}`",
                        value.kind(),
                        set.kind()
                    ),
                    doc.name,
                    line_number,
                );
            }

            Ok(())
        }
    }

    pub fn get_kind_with_argument(
        &mut self,
        name: &str,
        line_number: usize,
        component_definition_name_with_arguments: &Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<(
            fastn_resolved::PropertyValueSource,
            fastn_resolved::KindData,
            bool,
        )>,
    > {
        let name = ftd_p1::AccessModifier::remove_modifiers(name);
        let name = name
            .strip_prefix(ftd::interpreter::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter::utils::CLONE))
            .unwrap_or(name.as_str());

        let initial_kind_with_remaining_and_source =
            ftd::interpreter::utils::get_argument_for_reference_and_remaining(
                name,
                self,
                component_definition_name_with_arguments,
                loop_object_name_and_kind,
                line_number,
            )?
            .map(|v| (v.0.kind.to_owned(), v.1, v.2, v.0.mutable));

        let (initial_kind, remaining, source, mutable) =
            if let Some(r) = initial_kind_with_remaining_and_source {
                r
            } else {
                let (initial_thing, remaining) =
                    try_ok_state!(self.search_initial_thing(name, line_number)?);

                let (initial_kind, mutable) = match initial_thing {
                    ftd::interpreter::Thing::Record(r) => (
                        fastn_resolved::Kind::record(r.name.as_str())
                            .into_kind_data()
                            .caption_or_body(),
                        false,
                    ),
                    ftd::interpreter::Thing::OrType(o) => {
                        if let Some(remaining) = &remaining {
                            (
                                fastn_resolved::Kind::or_type_with_variant(
                                    o.name.as_str(),
                                    remaining.as_str(),
                                    format!("{}.{}", &o.name, remaining).as_str(),
                                )
                                .into_kind_data()
                                .caption_or_body(),
                                false,
                            )
                        } else {
                            (
                                fastn_resolved::Kind::or_type(o.name.as_str())
                                    .into_kind_data()
                                    .caption_or_body(),
                                false,
                            )
                        }
                    }
                    ftd::interpreter::Thing::OrTypeWithVariant { or_type, variant } => (
                        fastn_resolved::Kind::or_type_with_variant(
                            or_type.as_str(),
                            variant.name().as_str(),
                            variant.name().as_str(),
                        )
                        .into_kind_data()
                        .caption_or_body(),
                        false,
                    ),
                    ftd::interpreter::Thing::Variable(v) => (v.kind, v.mutable),
                    ftd::interpreter::Thing::Component(c) => (
                        fastn_resolved::Kind::ui_with_name(c.name.as_str())
                            .into_kind_data()
                            .caption_or_body(),
                        false,
                    ),
                    ftd::interpreter::Thing::WebComponent(c) => (
                        fastn_resolved::Kind::web_ui_with_name(c.name.as_str())
                            .into_kind_data()
                            .caption_or_body(),
                        false,
                    ),
                    ftd::interpreter::Thing::Function(f) => (f.return_kind, false),
                    ftd::interpreter::Thing::Export { .. } => unreachable!(),
                };

                (
                    initial_kind,
                    remaining,
                    fastn_resolved::PropertyValueSource::Global,
                    mutable,
                )
            };

        if let Some(remaining) = remaining
            && !initial_kind.is_module()
            && !initial_kind
                .kind
                .is_or_type_with_variant(&initial_kind.kind.get_name(), remaining.as_str())
        {
            return Ok(ftd::interpreter::StateWithThing::new_thing((
                source,
                try_ok_state!(get_kind_(
                    initial_kind.kind,
                    remaining.as_str(),
                    self,
                    line_number
                )?),
                mutable,
            )));
        }

        return Ok(ftd::interpreter::StateWithThing::new_thing((
            source,
            initial_kind,
            mutable,
        )));

        fn get_kind_(
            kind: fastn_resolved::Kind,
            name: &str,
            doc: &mut ftd::interpreter::TDoc,
            line_number: usize,
        ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::KindData>>
        {
            let (v, remaining) = ftd::interpreter::utils::split_at(name, ".");
            match kind {
                fastn_resolved::Kind::Record { name: rec_name } => {
                    let record = try_ok_state!(doc.search_record(rec_name.as_str(), line_number)?);
                    let field_kind = record.get_field(&v, doc.name, line_number)?.kind.to_owned();
                    if let Some(remaining) = remaining {
                        get_kind_(field_kind.kind, &remaining, doc, line_number)
                    } else {
                        Ok(ftd::interpreter::StateWithThing::new_thing(field_kind))
                    }
                }
                fastn_resolved::Kind::List { kind } => {
                    if let Some(remaining) = remaining {
                        get_kind_(*kind, &remaining, doc, line_number)
                    } else {
                        Ok(ftd::interpreter::StateWithThing::new_thing(
                            fastn_resolved::KindData::new(*kind),
                        ))
                    }
                }
                fastn_resolved::Kind::Optional { kind } => {
                    let state_with_thing = get_kind_(*kind, name, doc, line_number)?;
                    if let ftd::interpreter::StateWithThing::Thing(ref t) = state_with_thing {
                        Ok(ftd::interpreter::StateWithThing::new_thing(
                            t.to_owned().into_optional(),
                        ))
                    } else {
                        Ok(state_with_thing)
                    }
                }
                fastn_resolved::Kind::KwArgs => Ok(ftd::interpreter::StateWithThing::new_thing(
                    fastn_resolved::KindData::new(fastn_resolved::Kind::String),
                )),
                t => ftd::interpreter::utils::e2(
                    format!("Expected Record field `{name}`, found: `{t:?}`"),
                    doc.name,
                    line_number,
                ),
            }
        }
    }

    pub fn get_kind(
        &mut self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::KindData>> {
        match self.get_kind_with_argument(name, line_number, &None, &None)? {
            ftd::interpreter::StateWithThing::State(s) => {
                Ok(ftd::interpreter::StateWithThing::new_state(*s))
            }
            ftd::interpreter::StateWithThing::Continue => {
                Ok(ftd::interpreter::StateWithThing::new_continue())
            }
            ftd::interpreter::StateWithThing::Thing(fields) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(fields.1))
            }
        }
    }

    pub fn get_component(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::ComponentDefinition> {
        match self.get_thing(name, line_number)? {
            ftd::interpreter::Thing::Component(c) => Ok(c),
            t => self.err(
                format!("Expected Component, found: `{t:?}`").as_str(),
                name,
                "get_component",
                line_number,
            ),
        }
    }

    pub fn get_web_component(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::WebComponentDefinition> {
        match self.get_thing(name, line_number)? {
            ftd::interpreter::Thing::WebComponent(c) => Ok(c),
            t => self.err(
                format!("Expected web-component, found: `{t:?}`").as_str(),
                name,
                "get_web_component",
                line_number,
            ),
        }
    }

    pub fn search_component(
        &mut self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<fastn_resolved::ComponentDefinition>,
    > {
        match self.search_thing(name, line_number)? {
            ftd::interpreter::StateWithThing::State(s) => {
                Ok(ftd::interpreter::StateWithThing::new_state(*s))
            }
            ftd::interpreter::StateWithThing::Continue => {
                Ok(ftd::interpreter::StateWithThing::new_continue())
            }
            ftd::interpreter::StateWithThing::Thing(ftd::interpreter::Thing::Component(c)) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(c))
            }
            ftd::interpreter::StateWithThing::Thing(t) => self.err(
                format!("Expected Component, found: `{t:?}`").as_str(),
                name,
                "search_component",
                line_number,
            ),
        }
    }

    pub fn search_web_component(
        &mut self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<fastn_resolved::WebComponentDefinition>,
    > {
        match self.search_thing(name, line_number)? {
            ftd::interpreter::StateWithThing::State(s) => {
                Ok(ftd::interpreter::StateWithThing::new_state(*s))
            }
            ftd::interpreter::StateWithThing::Continue => {
                Ok(ftd::interpreter::StateWithThing::new_continue())
            }
            ftd::interpreter::StateWithThing::Thing(ftd::interpreter::Thing::WebComponent(c)) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(c))
            }
            ftd::interpreter::StateWithThing::Thing(t) => self.err(
                format!("Expected WebComponent, found: `{t:?}`").as_str(),
                name,
                "search_web_component",
                line_number,
            ),
        }
    }

    pub fn search_or_type(
        &mut self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::OrType>> {
        match self.search_thing(name, line_number)? {
            ftd::interpreter::StateWithThing::State(s) => {
                Ok(ftd::interpreter::StateWithThing::new_state(*s))
            }
            ftd::interpreter::StateWithThing::Continue => {
                Ok(ftd::interpreter::StateWithThing::new_continue())
            }
            ftd::interpreter::StateWithThing::Thing(ftd::interpreter::Thing::OrType(c)) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(c))
            }
            ftd::interpreter::StateWithThing::Thing(t) => self.err(
                format!("Expected OrType, found: `{t:?}`").as_str(),
                name,
                "search_or_type",
                line_number,
            ),
        }
    }

    pub fn search_or_type_with_variant(
        &mut self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<(String, fastn_resolved::OrTypeVariant)>,
    > {
        match self.search_thing(name, line_number)? {
            ftd::interpreter::StateWithThing::State(s) => {
                Ok(ftd::interpreter::StateWithThing::new_state(*s))
            }
            ftd::interpreter::StateWithThing::Continue => {
                Ok(ftd::interpreter::StateWithThing::new_continue())
            }
            ftd::interpreter::StateWithThing::Thing(
                ftd::interpreter::Thing::OrTypeWithVariant { or_type, variant },
            ) => Ok(ftd::interpreter::StateWithThing::new_thing((
                or_type, variant,
            ))),
            ftd::interpreter::StateWithThing::Thing(t) => self.err(
                format!("Expected OrTypeWithVariant, found: `{t:?}`").as_str(),
                name,
                "search_or_type_with_variant",
                line_number,
            ),
        }
    }

    pub fn get_thing(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::Thing> {
        let name = name
            .strip_prefix(ftd::interpreter::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter::utils::CLONE))
            .unwrap_or(name);

        let (initial_thing, remaining) = self.get_initial_thing(name, line_number)?;

        if let Some(remaining) = remaining {
            return get_thing_(self, line_number, remaining.as_str(), initial_thing);
        }
        return Ok(initial_thing);

        fn get_thing_(
            doc: &ftd::interpreter::TDoc,
            line_number: usize,
            name: &str,
            thing: ftd::interpreter::Thing,
        ) -> ftd::interpreter::Result<ftd::interpreter::Thing> {
            use ftd::interpreter::PropertyValueExt;
            use itertools::Itertools;

            let (v, remaining) = ftd::interpreter::utils::split_at(name, ".");
            let thing = match thing.clone() {
                ftd::interpreter::Thing::Variable(fastn_resolved::Variable {
                    name,
                    value,
                    mutable,
                    ..
                }) => {
                    let value_kind = value.kind();
                    let fields = match value.resolve(doc, line_number)?.inner() {
                        Some(fastn_resolved::Value::Record { fields, .. }) => fields,
                        Some(fastn_resolved::Value::Object { values }) => values,
                        Some(fastn_resolved::Value::KwArgs { arguments }) => arguments,
                        Some(fastn_resolved::Value::List { data, .. }) => data
                            .into_iter()
                            .enumerate()
                            .map(|(index, v)| (index.to_string(), v))
                            .collect::<ftd::Map<fastn_resolved::PropertyValue>>(),
                        None => {
                            let kind_name = match value_kind.get_record_name() {
                                Some(name) => name,
                                _ => {
                                    return doc.err(
                                        "not an record",
                                        thing,
                                        "get_thing",
                                        line_number,
                                    );
                                }
                            };
                            let kind_thing = doc.get_thing(kind_name, line_number)?;
                            let kind = match kind_thing
                                .record(doc.name, line_number)?
                                .fields
                                .iter()
                                .find(|f| f.name.eq(&v))
                                .map(|v| v.kind.to_owned())
                            {
                                Some(f) => f,
                                _ => {
                                    return doc.err(
                                        "not an record or or-type",
                                        thing,
                                        "get_thing",
                                        line_number,
                                    );
                                }
                            };
                            let thing =
                                ftd::interpreter::Thing::Variable(fastn_resolved::Variable {
                                    name,
                                    kind: kind.to_owned(),
                                    mutable,
                                    value: fastn_resolved::PropertyValue::Value {
                                        value: fastn_resolved::Value::Optional {
                                            data: Box::new(None),
                                            kind,
                                        },
                                        is_mutable: mutable,
                                        line_number,
                                    },
                                    conditional_value: vec![],
                                    line_number,
                                    is_static: !mutable,
                                });
                            if let Some(remaining) = remaining {
                                return get_thing_(doc, line_number, &remaining, thing);
                            }
                            return Ok(thing);
                        }
                        _ => return doc.err("not an record", thing, "get_thing", line_number),
                    };
                    match fields.get(&v) {
                        Some(fastn_resolved::PropertyValue::Value {
                            value: val,
                            line_number,
                            is_mutable,
                        }) => ftd::interpreter::Thing::Variable(fastn_resolved::Variable {
                            name,
                            kind: fastn_resolved::KindData {
                                kind: val.kind(),
                                caption: false,
                                body: false,
                            },
                            mutable: false,
                            value: fastn_resolved::PropertyValue::Value {
                                value: val.to_owned(),
                                line_number: *line_number,
                                is_mutable: *is_mutable,
                            },
                            conditional_value: vec![],
                            line_number: *line_number,
                            is_static: !mutable,
                        }),
                        Some(fastn_resolved::PropertyValue::Reference { name, .. })
                        | Some(fastn_resolved::PropertyValue::Clone { name, .. }) => {
                            let (initial_thing, name) = doc.get_initial_thing(name, line_number)?;
                            if let Some(remaining) = name {
                                get_thing_(doc, line_number, remaining.as_str(), initial_thing)?
                            } else {
                                initial_thing
                            }
                        }
                        _ => thing.clone(),
                    }
                }
                ftd::interpreter::Thing::OrType(fastn_resolved::OrType {
                    name, variants, ..
                }) => {
                    let variant = variants
                        .iter()
                        .find_or_first(|variant| variant.name().eq(&format!("{name}.{v}")))
                        .ok_or(ftd::interpreter::Error::ParseError {
                            message: format!("Cant't find `{v}` variant in `{name}` or-type"),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?;
                    variant.to_thing(doc.name, line_number)?
                }
                _ => {
                    return doc.err("not an or-type", thing, "get_thing", line_number);
                }
            };
            if let Some(remaining) = remaining {
                return get_thing_(doc, line_number, &remaining, thing);
            }
            Ok(thing)
        }
    }

    pub fn get_function(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Function> {
        let initial_thing = self.get_initial_thing(name, line_number)?.0;
        Ok(initial_thing.function(self.name, line_number)?.clone())
    }

    pub fn search_function(
        &mut self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::Function>> {
        let name = ftd_p1::AccessModifier::remove_modifiers(name);
        let initial_thing = try_ok_state!(self.search_initial_thing(name.as_str(), line_number)?).0;
        Ok(ftd::interpreter::StateWithThing::new_thing(
            initial_thing.function(self.name, line_number)?.clone(),
        ))
    }

    pub fn get_initial_variable(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter::Result<(fastn_resolved::Variable, Option<String>)> {
        self.get_initial_variable_with_inherited(name, line_number, &Default::default())
    }

    pub fn get_initial_variable_with_inherited(
        &'a self,
        name: &'a str,
        line_number: usize,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<(fastn_resolved::Variable, Option<String>)> {
        let (initial_thing, remaining) =
            self.get_initial_thing_with_inherited(name, line_number, inherited_variables)?;
        Ok((initial_thing.variable(self.name, line_number)?, remaining))
    }

    pub fn scan_thing(&mut self, name: &str, line_number: usize) -> ftd::interpreter::Result<()> {
        let name = name
            .strip_prefix(ftd::interpreter::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter::utils::CLONE))
            .unwrap_or(name);

        self.scan_initial_thing(name, line_number)
    }

    pub fn scan_initial_thing_from_doc_name(
        &mut self,
        doc_name: String,
        thing_name: String,
        remaining: Option<String>,
        line_number: usize,
        exports: Vec<String>,
        caller: String,
    ) -> ftd::interpreter::Result<()> {
        use itertools::Itertools;

        let name = format!(
            "{}#{}{}",
            doc_name,
            thing_name,
            remaining
                .as_ref()
                .map(|v| format!(".{v}"))
                .unwrap_or_default()
        );

        let state = if let Some(state) = {
            match &mut self.bag {
                BagOrState::Bag(_) => None,
                BagOrState::State(s) => Some(s),
            }
        } {
            state
        } else {
            return self.err("not found", name, "search_thing", line_number);
        };

        if doc_name.eq(ftd::interpreter::FTD_INHERITED) {
            return Ok(());
        }

        // let current_parsed_document = state.parsed_libs.get(self.name).unwrap();

        /*if doc_name.ne(self.name) {
            let current_doc_contains_thing = current_parsed_document
                .ast
                .iter()
                .filter(|v| {
                    !v.is_component()
                        && (v.name().eq(&format!("{}.{}", doc_name, thing_name))
                            || v.name()
                                .starts_with(format!("{}.{}.", doc_name, thing_name).as_str()))
                })
                .map(|v| (0, v.to_owned()))
                .collect_vec();
            if !current_doc_contains_thing.is_empty()
                && !state.to_process.contains.contains(&(
                    self.name.to_string(),
                    format!("{}#{}", doc_name, thing_name),
                ))
            {
                state
                    .to_process
                    .stack
                    .push((self.name.to_string(), current_doc_contains_thing));
                state.to_process.contains.insert((
                    self.name.to_string(),
                    format!("{}#{}", doc_name, thing_name),
                ));
            }
        }*/

        if let Some(parsed_document) = state.parsed_libs.get(doc_name.as_str()) {
            let ast_for_thing = parsed_document
                .ast
                .iter()
                .filter(|v| {
                    !v.is_component_invocation()
                        && (v.name().eq(thing_name.as_str())
                            || v.name().starts_with(format!("{thing_name}.").as_str()))
                })
                .map(|v| (0, v.to_owned()))
                .collect_vec();

            if ast_for_thing.is_empty() {
                if parsed_document
                    .foreign_variable
                    .iter()
                    .any(|v| thing_name.eq(v))
                {
                    state
                        .pending_imports
                        .stack
                        .push(ftd::interpreter::PendingImportItem {
                            module: doc_name.to_string(),
                            thing_name: name,
                            line_number,
                            caller,
                            exports,
                        });
                    state
                        .pending_imports
                        .contains
                        .insert((doc_name.to_string(), format!("{doc_name}#{thing_name}")));
                } else if doc_name.ne(&caller)
                    && parsed_document
                        .re_exports
                        .module_things
                        .contains_key(thing_name.as_str())
                {
                    let module = parsed_document
                        .re_exports
                        .module_things
                        .get(thing_name.as_str())
                        .cloned()
                        .unwrap();
                    let mut exports = exports;
                    exports.push(name);
                    return self.scan_initial_thing_from_doc_name(
                        module,
                        thing_name,
                        remaining,
                        line_number,
                        exports,
                        doc_name,
                    );
                } else if doc_name.eq(&caller)
                    && parsed_document.exposings.contains_key(thing_name.as_str())
                {
                    let module = parsed_document
                        .exposings
                        .get(thing_name.as_str())
                        .cloned()
                        .unwrap();
                    let mut exports = exports;
                    exports.push(name);
                    return self.scan_initial_thing_from_doc_name(
                        module,
                        thing_name,
                        remaining,
                        line_number,
                        exports,
                        doc_name,
                    );
                }

                /*for module in parsed_document.re_exports.all_things.clone() {
                    if let Ok(()) = self.scan_initial_thing_from_doc_name(
                        module,
                        thing_name.to_string(),
                        remaining.clone(),
                        line_number,
                    ) {
                        return Ok(());
                    }
                }*/

                return Ok(());
            }

            if !state
                .pending_imports
                .contains
                .contains(&(doc_name.to_string(), format!("{doc_name}#{thing_name}")))
            {
                state
                    .pending_imports
                    .contains
                    .insert((doc_name.to_string(), format!("{doc_name}#{thing_name}")));

                state
                    .pending_imports
                    .stack
                    .push(ftd::interpreter::PendingImportItem {
                        module: doc_name.to_string(),
                        thing_name: name,
                        line_number,
                        caller,
                        exports,
                    });
            }
        } else {
            state
                .pending_imports
                .stack
                .push(ftd::interpreter::PendingImportItem {
                    module: doc_name.to_string(),
                    thing_name: name,
                    line_number,
                    caller,
                    exports,
                });
            state
                .pending_imports
                .contains
                .insert((doc_name.to_string(), format!("{doc_name}#{thing_name}")));
        }

        Ok(())
    }

    pub fn scan_initial_thing(
        &mut self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<()> {
        let name = name
            .strip_prefix(ftd::interpreter::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter::utils::CLONE))
            .unwrap_or(name);

        if self.get_initial_thing(name, line_number).is_ok() {
            return Ok(());
        }

        let name = self.resolve_name(name);

        let (doc_name, thing_name, remaining) = // Todo: use remaining
            ftd::interpreter::utils::get_doc_name_and_thing_name_and_remaining(
                name.as_str(),
                self.name,
                line_number,
            );

        self.scan_initial_thing_from_doc_name(
            doc_name,
            thing_name,
            remaining,
            line_number,
            vec![],
            self.name.to_string(),
        )
    }

    pub fn search_thing(
        &mut self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ftd::interpreter::Thing>> {
        let name = ftd_p1::AccessModifier::remove_modifiers(name);
        let name = name
            .strip_prefix(ftd::interpreter::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter::utils::CLONE))
            .unwrap_or(name.as_str());

        let (initial_thing, remaining) =
            try_ok_state!(self.search_initial_thing(name, line_number)?);

        if let Some(remaining) = remaining {
            return search_thing_(self, line_number, remaining.as_str(), initial_thing);
        }
        return Ok(ftd::interpreter::StateWithThing::new_thing(initial_thing));

        fn search_thing_(
            doc: &mut ftd::interpreter::TDoc,
            line_number: usize,
            name: &str,
            thing: ftd::interpreter::Thing,
        ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ftd::interpreter::Thing>>
        {
            use ftd::interpreter::PropertyValueExt;

            let (v, remaining) = ftd::interpreter::utils::split_at(name, ".");
            let thing = match thing.clone() {
                ftd::interpreter::Thing::Variable(fastn_resolved::Variable {
                    name,
                    value,
                    mutable,
                    ..
                }) => {
                    let value_kind = value.kind();
                    let fields = match value.resolve(doc, line_number)?.inner() {
                        Some(fastn_resolved::Value::Record { fields, .. }) => fields,
                        Some(fastn_resolved::Value::Object { values }) => values,
                        Some(fastn_resolved::Value::KwArgs { arguments }) => arguments,
                        Some(fastn_resolved::Value::List { data, .. }) => data
                            .into_iter()
                            .enumerate()
                            .map(|(index, v)| (index.to_string(), v))
                            .collect::<ftd::Map<fastn_resolved::PropertyValue>>(),
                        None => {
                            let kind_name = match value_kind.get_record_name() {
                                Some(name) => name,
                                _ => {
                                    return doc.err(
                                        "not an record",
                                        thing,
                                        "search_thing_",
                                        line_number,
                                    );
                                }
                            };
                            let kind_thing =
                                try_ok_state!(doc.search_thing(kind_name, line_number)?);
                            let kind = match kind_thing
                                .record(doc.name, line_number)?
                                .fields
                                .iter()
                                .find(|f| f.name.eq(&v))
                                .map(|v| v.kind.to_owned())
                            {
                                Some(f) => f,
                                _ => {
                                    return doc.err(
                                        "not an record or or-type",
                                        thing,
                                        "search_thing_",
                                        line_number,
                                    );
                                }
                            };
                            let thing =
                                ftd::interpreter::Thing::Variable(fastn_resolved::Variable {
                                    name,
                                    kind: kind.to_owned(),
                                    mutable,
                                    value: fastn_resolved::PropertyValue::Value {
                                        value: fastn_resolved::Value::Optional {
                                            data: Box::new(None),
                                            kind,
                                        },
                                        is_mutable: mutable,
                                        line_number,
                                    },
                                    conditional_value: vec![],
                                    line_number,
                                    is_static: !mutable,
                                });
                            if let Some(remaining) = remaining {
                                return search_thing_(doc, line_number, &remaining, thing);
                            }
                            return Ok(ftd::interpreter::StateWithThing::new_thing(thing));
                        }
                        _ => return doc.err("not an record", thing, "search_thing_", line_number),
                    };
                    match fields.get(&v) {
                        Some(fastn_resolved::PropertyValue::Value {
                            value: val,
                            line_number,
                            ..
                        }) => ftd::interpreter::Thing::Variable(fastn_resolved::Variable {
                            name,
                            kind: fastn_resolved::KindData {
                                kind: val.kind(),
                                caption: false,
                                body: false,
                            },
                            mutable,
                            value: fastn_resolved::PropertyValue::Value {
                                value: val.to_owned(),
                                line_number: *line_number,
                                is_mutable: mutable,
                            },
                            conditional_value: vec![],
                            line_number: *line_number,
                            is_static: !mutable,
                        }),
                        property_value @ Some(fastn_resolved::PropertyValue::Reference {
                            name,
                            ..
                        })
                        | property_value @ Some(fastn_resolved::PropertyValue::Clone {
                            name, ..
                        }) => {
                            let name = ftd_p1::AccessModifier::remove_modifiers(name);
                            let (initial_thing, name) = try_ok_state!(
                                doc.search_initial_thing(name.as_str(), line_number)?
                            );

                            let mut thing = if let Some(remaining) = name {
                                try_ok_state!(search_thing_(
                                    doc,
                                    line_number,
                                    remaining.as_str(),
                                    initial_thing,
                                )?)
                            } else {
                                initial_thing
                            };

                            if property_value.unwrap().is_clone()
                                && let Ok(mut variable) =
                                    thing.clone().variable(doc.name, thing.line_number())
                            {
                                variable.mutable = mutable;
                                thing = ftd::interpreter::Thing::Variable(variable);
                            }

                            thing
                        }
                        _ => thing,
                    }
                }
                ftd::interpreter::Thing::OrType(fastn_resolved::OrType {
                    name, variants, ..
                }) => {
                    let or_type_name = fastn_resolved::OrType::or_type_name(name.as_str());
                    if let Some(thing) = variants.into_iter().find(|or_type_variant| {
                        let variant_name = or_type_variant.name();
                        variant_name
                            .trim_start_matches(format!("{or_type_name}.").as_str())
                            .eq(&v)
                    }) {
                        // Todo: Handle remaining
                        ftd::interpreter::Thing::OrTypeWithVariant {
                            or_type: name.clone(),
                            variant: thing,
                        }
                    } else {
                        return doc.err(
                            format!("Can't find variant `{name}` in or-type `{or_type_name}`")
                                .as_str(),
                            thing,
                            "search_thing_",
                            line_number,
                        );
                    }
                }
                _ => {
                    return doc.err(
                        format!("not an or-type `{name}`").as_str(),
                        thing,
                        "search_thing_",
                        line_number,
                    );
                }
            };
            if let Some(remaining) = remaining {
                return search_thing_(doc, line_number, &remaining, thing);
            }
            Ok(ftd::interpreter::StateWithThing::new_thing(thing))
        }
    }

    pub fn search_initial_thing_from_doc_name(
        &mut self,
        doc_name: String,
        thing_name: String,
        remaining: Option<String>,
        line_number: usize,
        caller: &str,
        exports: Vec<String>,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<(ftd::interpreter::Thing, Option<String>)>,
    > {
        use itertools::Itertools;

        let thing_name = ftd_p1::AccessModifier::remove_modifiers(thing_name.as_str());
        let name = format!(
            "{}#{}{}",
            doc_name,
            thing_name,
            remaining
                .as_ref()
                .map(|v| format!(".{v}"))
                .unwrap_or_default()
        );

        let state = if let Some(state) = {
            match &mut self.bag {
                BagOrState::Bag(_) => None,
                BagOrState::State(s) => Some(s),
            }
        } {
            state
        } else {
            return self.err("not found", name, "search_thing", line_number);
        };

        let current_parsed_document = state.parsed_libs.get(state.id.as_str()).unwrap();

        if doc_name.ne(state.id.as_str()) {
            let current_doc_contains_thing = current_parsed_document
                .ast
                .iter()
                .filter(|v| {
                    let name = ftd::interpreter::utils::resolve_name(
                        v.name().as_str(),
                        state.id.as_str(),
                        &current_parsed_document.doc_aliases,
                    );
                    !v.is_component_invocation()
                        && (name.eq(&format!("{doc_name}#{thing_name}"))
                            || name.starts_with(format!("{doc_name}#{thing_name}.").as_str()))
                })
                .map(|v| ftd::interpreter::ToProcessItem {
                    number_of_scan: 0,
                    ast: v.to_owned(),
                    exports: exports.clone(),
                })
                .collect_vec();
            if !current_doc_contains_thing.is_empty() {
                state
                    .to_process
                    .stack
                    .push((state.id.to_string(), current_doc_contains_thing));

                if !state
                    .to_process
                    .contains
                    .contains(&(state.id.to_string(), format!("{doc_name}#{thing_name}")))
                {
                    state
                        .to_process
                        .contains
                        .insert((state.id.to_string(), format!("{doc_name}#{thing_name}")));
                }
            } else if !current_doc_contains_thing.is_empty() && state.peek_stack().unwrap().1.gt(&4)
            {
                return self.err("not found", name, "search_thing", line_number);
            }
        }

        if let Some(parsed_document) = state.parsed_libs.get(doc_name.as_str()) {
            let ast_for_thing = parsed_document
                .ast
                .iter()
                .filter(|v| {
                    !v.is_component_invocation()
                        && (v.name().eq(&thing_name)
                            || v.name().starts_with(format!("{thing_name}.").as_str()))
                })
                .map(|v| ftd::interpreter::ToProcessItem {
                    number_of_scan: 0,
                    ast: v.to_owned(),
                    exports: exports.clone(),
                })
                .collect_vec();

            if ast_for_thing.is_empty() {
                if parsed_document
                    .foreign_variable
                    .iter()
                    .any(|v| thing_name.eq(v))
                {
                    return Ok(ftd::interpreter::StateWithThing::new_state(
                        ftd::interpreter::InterpreterWithoutState::StuckOnForeignVariable {
                            module: doc_name.to_string(),
                            variable: remaining
                                .map(|v| format!("{thing_name}.{v}"))
                                .unwrap_or(thing_name),
                            caller_module: self.name.to_string(),
                        },
                    ));
                }

                if doc_name.ne(&caller)
                    && parsed_document
                        .re_exports
                        .module_things
                        .contains_key(thing_name.as_str())
                {
                    let module = parsed_document
                        .re_exports
                        .module_things
                        .get(thing_name.as_str())
                        .cloned()
                        .unwrap();
                    let mut exports = exports;
                    exports.push(name);
                    return self.search_initial_thing_from_doc_name(
                        module,
                        thing_name,
                        remaining,
                        line_number,
                        doc_name.as_str(),
                        exports,
                    );
                } else if doc_name.ne(&caller) && !parsed_document.re_exports.all_things.is_empty()
                {
                    if parsed_document.re_exports.all_things.len() != 1 {
                        return self.err(
                            "Currently, fastn only support one * export",
                            name,
                            "search_thing",
                            line_number,
                        );
                    }
                    let module = parsed_document
                        .re_exports
                        .all_things
                        .first()
                        .unwrap()
                        .clone();
                    let mut exports = exports;
                    exports.push(name);
                    return self.search_initial_thing_from_doc_name(
                        module,
                        thing_name,
                        remaining,
                        line_number,
                        doc_name.as_str(),
                        exports,
                    );
                } else if doc_name.eq(&caller)
                    && parsed_document.exposings.contains_key(thing_name.as_str())
                {
                    let module = parsed_document
                        .exposings
                        .get(thing_name.as_str())
                        .cloned()
                        .unwrap();
                    let mut exports = exports;
                    exports.push(name);
                    return self.search_initial_thing_from_doc_name(
                        module,
                        thing_name,
                        remaining,
                        line_number,
                        doc_name.as_str(),
                        exports,
                    );
                }

                /*for module in parsed_document.re_exports.all_things.clone() {
                    if let Ok(thing) = self.search_initial_thing_from_doc_name(
                        module,
                        thing_name.clone(),
                        remaining.clone(),
                        line_number,
                    ) {
                        return Ok(thing);
                    }
                }*/
                return self.err("not found", name, "search_thing", line_number);
            }

            state
                .to_process
                .stack
                .push((doc_name.to_string(), ast_for_thing));
            if !state
                .to_process
                .contains
                .contains(&(doc_name.to_string(), format!("{doc_name}#{thing_name}")))
            {
                state
                    .to_process
                    .contains
                    .insert((doc_name.to_string(), format!("{doc_name}#{thing_name}")));
            }

            return Ok(ftd::interpreter::StateWithThing::new_continue());
        }

        if doc_name.eq(self.name) {
            return self.err("not found", name, "search_thing", line_number);
        }

        state
            .pending_imports
            .stack
            .push(ftd::interpreter::PendingImportItem {
                module: doc_name.to_string(),
                thing_name: name,
                line_number,
                caller: caller.to_string(),
                exports,
            });

        Ok(ftd::interpreter::StateWithThing::new_state(
            ftd::interpreter::InterpreterWithoutState::StuckOnImport {
                module: doc_name,
                caller_module: caller.to_string(),
            },
        ))
    }

    pub fn search_initial_thing(
        &mut self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<(ftd::interpreter::Thing, Option<String>)>,
    > {
        let name = name
            .strip_prefix(ftd::interpreter::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter::utils::CLONE))
            .unwrap_or(name);

        if let Ok(thing) = self.get_initial_thing(name, line_number) {
            return Ok(ftd::interpreter::StateWithThing::new_thing(thing));
        }

        let name = self.resolve_name(name);

        let (doc_name, thing_name, remaining) = // Todo: use remaining
            ftd::interpreter::utils::get_doc_name_and_thing_name_and_remaining(
                name.as_str(),
                self.name,
                line_number,
            );

        self.search_initial_thing_from_doc_name(
            doc_name,
            thing_name,
            remaining,
            line_number,
            self.name,
            vec![],
        )
    }

    pub fn get_initial_thing(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter::Result<(ftd::interpreter::Thing, Option<String>)> {
        self.get_initial_thing_with_inherited(name, line_number, &Default::default())
    }

    pub fn get_initial_thing_with_inherited(
        &'a self,
        name: &'a str,
        line_number: usize,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<(ftd::interpreter::Thing, Option<String>)> {
        let name = name
            .strip_prefix(ftd::interpreter::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter::utils::CLONE))
            .unwrap_or(name);

        if let Some(name) =
            ftd::interpreter::utils::find_inherited_variables(name, inherited_variables, None)
        {
            return self.get_initial_thing_with_inherited(
                name.as_str(),
                line_number,
                inherited_variables,
            );
        }

        let name = self.resolve_name(name);

        self.get_reexport_thing(&name, line_number)
            .map(|(thing, remaining)| (thing.clone(), remaining))
    }

    fn get_reexport_thing(
        &self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<(&ftd::interpreter::Thing, Option<String>)> {
        let (splited_name, remaining_value) = if let Ok(function_name) =
            ftd::interpreter::utils::get_function_name(name, self.name, line_number)
        {
            (function_name, None)
        } else {
            ftd::interpreter::utils::get_doc_name_and_remaining(name, self.name, line_number)
        };

        let (thing_name, remaining) = match self.bag().get(splited_name.as_str()) {
            Some(a) => (a, remaining_value),
            None => match self.bag().get(name).map(|v| (v, None)) {
                Some(a) => a,
                None => {
                    return self.err("not found", splited_name, "get_initial_thing", line_number);
                }
            },
        };

        if let ftd::interpreter::Thing::Export { from, .. } = thing_name {
            let thing_name = self.get_reexport_thing(from, line_number)?.0;
            return Ok((thing_name, remaining));
        }

        Ok((thing_name, remaining))
    }

    pub fn rows_to_value(
        &self,
        rows: &[Vec<serde_json::Value>],
        kind: &fastn_resolved::Kind,
        value: &ftd_ast::VariableValue,
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
        Ok(match kind {
            fastn_resolved::Kind::List { kind, .. } => {
                let mut data = vec![];
                for row in rows {
                    data.push(
                        self.row_to_value(row, kind, value)?
                            .into_property_value(false, value.line_number()),
                    );
                }

                fastn_resolved::Value::List {
                    data,
                    kind: kind.to_owned().into_kind_data(),
                }
            }
            t => unimplemented!(
                "{:?} not yet implemented, line number: {}, doc: {}",
                t,
                value.line_number(),
                self.name.to_string()
            ),
        })
    }

    fn row_to_record(
        &self,
        row: &[serde_json::Value],
        name: &str,
        value: &ftd_ast::VariableValue,
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
        let rec = self.get_record(name, value.line_number())?;
        let rec_fields = rec.fields;
        let mut fields: ftd::Map<fastn_resolved::PropertyValue> = Default::default();
        for (idx, key) in rec_fields.iter().enumerate() {
            let val = match row.get(idx) {
                Some(v) => v,
                None => {
                    return ftd::interpreter::utils::e2(
                        format!("key not found: {}", key.name.as_str()),
                        self.name,
                        value.line_number(),
                    );
                }
            };
            fields.insert(
                key.name.to_string(),
                self.from_json(val, &key.kind.kind, value)?
                    .into_property_value(false, value.line_number()),
            );
        }

        Ok(fastn_resolved::Value::Record {
            name: name.to_string(),
            fields,
        })
    }

    pub fn row_to_value(
        &self,
        row: &[serde_json::Value],
        kind: &fastn_resolved::Kind,
        value: &ftd_ast::VariableValue,
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
        if let fastn_resolved::Kind::Record { name } = kind {
            return self.row_to_record(row, name, value);
        }

        if row.len() != 1 {
            return ftd::interpreter::utils::e2(
                format!("expected one column, found: {}", row.len()),
                self.name,
                value.line_number(),
            );
        }

        self.as_json_(
            kind,
            &row[0],
            value.caption(),
            value.record_name(),
            value.line_number(),
        )
    }

    pub fn from_json<T>(
        &self,
        json: &T,
        kind: &fastn_resolved::Kind,
        value: &ftd_ast::VariableValue,
    ) -> ftd::interpreter::Result<fastn_resolved::Value>
    where
        T: serde::Serialize + std::fmt::Debug,
    {
        let name = match value.inner() {
            Some(ftd_ast::VariableValue::Record { name, .. }) => Some(name),
            _ => None,
        };

        let json = serde_json::to_value(json).map_err(|e| ftd::interpreter::Error::ParseError {
            message: format!("Can't serialize to json: {e:?}, key={name:?}, found: {json:?}"),
            doc_id: self.name.to_string(),
            line_number: value.line_number(),
        })?;

        self.as_json_(
            kind,
            &json,
            value.caption(),
            value.record_name(),
            value.line_number(),
        )
    }

    fn handle_object(
        &self,
        kind: &fastn_resolved::Kind,
        o: &serde_json::Map<String, serde_json::Value>,
        default_value: Option<String>,
        record_name: Option<String>,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
        if let Some(name) = record_name {
            if let Some(v) = o.get(name.as_str()) {
                return self.as_json_(kind, v, default_value, None, line_number);
            } else if let Some(v) = default_value {
                return self.as_json_(kind, &serde_json::Value::String(v), None, None, line_number);
            }
        }

        ftd::interpreter::utils::e2(
            format!("Can't parse to {kind:?}, found: {o:?}"),
            self.name,
            line_number,
        )
    }

    fn as_json_(
        &self,
        kind: &fastn_resolved::Kind,
        json: &serde_json::Value,
        default_value: Option<String>,
        record_name: Option<String>,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
        Ok(match kind {
            fastn_resolved::Kind::String => fastn_resolved::Value::String {
                text: match json {
                    serde_json::Value::String(v) => v.to_string(),
                    serde_json::Value::Object(o) => {
                        return self.handle_object(
                            kind,
                            o,
                            default_value,
                            record_name,
                            line_number,
                        );
                    }
                    _ => {
                        return ftd::interpreter::utils::e2(
                            format!("Can't parse to string, found: {json}"),
                            self.name,
                            line_number,
                        );
                    }
                },
            },
            fastn_resolved::Kind::Integer => fastn_resolved::Value::Integer {
                value: match json {
                    serde_json::Value::Number(n) => {
                        n.as_i64()
                            .ok_or_else(|| ftd::interpreter::Error::ParseError {
                                message: format!("Can't parse to integer, found: {json}"),
                                doc_id: self.name.to_string(),
                                line_number,
                            })?
                    }
                    serde_json::Value::String(s) => {
                        s.parse::<i64>()
                            .map_err(|_| ftd::interpreter::Error::ParseError {
                                message: format!("Can't parse to integer, found: {json}"),
                                doc_id: self.name.to_string(),
                                line_number,
                            })?
                    }
                    serde_json::Value::Object(o) => {
                        return self.handle_object(
                            kind,
                            o,
                            default_value,
                            record_name,
                            line_number,
                        );
                    }
                    _ => {
                        return ftd::interpreter::utils::e2(
                            format!("Can't parse to integer, found: {json}"),
                            self.name,
                            line_number,
                        );
                    }
                },
            },
            fastn_resolved::Kind::Decimal => fastn_resolved::Value::Decimal {
                value: match json {
                    serde_json::Value::Number(n) => {
                        n.as_f64()
                            .ok_or_else(|| ftd::interpreter::Error::ParseError {
                                message: format!("Can't parse to decimal, found: {json}"),
                                doc_id: self.name.to_string(),
                                line_number,
                            })?
                    }
                    serde_json::Value::String(s) => {
                        s.parse::<f64>()
                            .map_err(|_| ftd::interpreter::Error::ParseError {
                                message: format!("Can't parse to decimal, found: {json}"),
                                doc_id: self.name.to_string(),
                                line_number,
                            })?
                    }
                    serde_json::Value::Object(o) => {
                        return self.handle_object(
                            kind,
                            o,
                            default_value,
                            record_name,
                            line_number,
                        );
                    }
                    _ => {
                        return ftd::interpreter::utils::e2(
                            format!("Can't parse to decimal, found: {json}"),
                            self.name,
                            line_number,
                        );
                    }
                },
            },
            fastn_resolved::Kind::Boolean => fastn_resolved::Value::Boolean {
                value: match json {
                    serde_json::Value::Bool(n) => *n,
                    serde_json::Value::String(s) => {
                        s.parse::<bool>()
                            .map_err(|_| ftd::interpreter::Error::ParseError {
                                message: format!("Can't parse to boolean, found: {json}"),
                                doc_id: self.name.to_string(),
                                line_number,
                            })?
                    }
                    serde_json::Value::Number(n) => match n.as_i64() {
                        Some(0) => false,
                        Some(1) => true,
                        _ => {
                            return Err(ftd::interpreter::Error::ParseError {
                                message: format!("Can't parse to decimal, found: {json}"),
                                doc_id: self.name.to_string(),
                                line_number,
                            });
                        }
                    },
                    serde_json::Value::Object(o) => {
                        return self.handle_object(
                            kind,
                            o,
                            default_value,
                            record_name,
                            line_number,
                        );
                    }
                    _ => {
                        return ftd::interpreter::utils::e2(
                            format!("Can't parse to boolean, found: {json}"),
                            self.name,
                            line_number,
                        );
                    }
                },
            },
            fastn_resolved::Kind::Record { name, .. } => {
                let rec_fields = self.get_record(name, line_number)?.fields;
                let mut fields: ftd::Map<fastn_resolved::PropertyValue> = Default::default();
                if let serde_json::Value::Object(o) = json {
                    for field in rec_fields {
                        let val = match o.get(&field.name) {
                            Some(v) => v.to_owned(),
                            None if field.kind.is_optional() => serde_json::Value::Null,
                            None if field.kind.is_list() => serde_json::Value::Array(vec![]),
                            None => {
                                return ftd::interpreter::utils::e2(
                                    format!("key not found: {}", field.name.as_str()),
                                    self.name,
                                    line_number,
                                );
                            }
                        };
                        fields.insert(
                            field.name,
                            fastn_resolved::PropertyValue::Value {
                                value: self.as_json_(
                                    &field.kind.kind,
                                    &val,
                                    if field.kind.caption {
                                        default_value.clone()
                                    } else {
                                        None
                                    },
                                    None,
                                    line_number,
                                )?,
                                is_mutable: false,
                                line_number,
                            },
                        );
                    }
                } else if let serde_json::Value::String(s) = json {
                    if let Some(field) = rec_fields.into_iter().find(|field| field.kind.caption) {
                        fields.insert(
                            field.name,
                            fastn_resolved::PropertyValue::Value {
                                value: fastn_resolved::Value::String {
                                    text: s.to_string(),
                                },
                                is_mutable: false,
                                line_number,
                            },
                        );
                    } else {
                        return ftd::interpreter::utils::e2(
                            format!("expected object of record type1 {name}, found: {json}"),
                            self.name,
                            line_number,
                        );
                    }
                } else {
                    // Todo: Handle default_value
                    return ftd::interpreter::utils::e2(
                        format!("expected object of record type2 {name}, found: {json}"),
                        self.name,
                        line_number,
                    );
                }
                fastn_resolved::Value::Record {
                    name: name.to_string(),
                    fields,
                }
            }
            fastn_resolved::Kind::List { kind, .. } => {
                let mut data: Vec<fastn_resolved::PropertyValue> = vec![];
                if let serde_json::Value::Array(list) = json {
                    for item in list {
                        data.push(fastn_resolved::PropertyValue::Value {
                            value: self.as_json_(kind, item, None, None, line_number)?,
                            is_mutable: false,
                            line_number,
                        });
                    }
                } else {
                    // Todo: Handle `default_value`
                    return ftd::interpreter::utils::e2(
                        format!("expected object of list type, found: {json}"),
                        self.name,
                        line_number,
                    );
                }
                fastn_resolved::Value::List {
                    data,
                    kind: kind.to_owned().into_kind_data(),
                }
            }
            fastn_resolved::Kind::Optional { kind, .. } => {
                let kind = kind.as_ref();
                match json {
                    serde_json::Value::Null => fastn_resolved::Value::Optional {
                        kind: kind.clone().into_kind_data(),
                        data: Box::new(None),
                    },
                    serde_json::Value::Object(o)
                        if record_name
                            .as_ref()
                            .map(|v| !o.contains_key(v))
                            .unwrap_or_default() =>
                    {
                        return Ok(fastn_resolved::Value::Optional {
                            kind: kind.clone().into_kind_data(),
                            data: Box::new(None),
                        });
                    }
                    _ => self.as_json_(kind, json, default_value, record_name, line_number)?,
                }
            }
            t => unimplemented!(
                "{:?} not yet implemented, line number: {}, doc: {}",
                t,
                line_number,
                self.name.to_string()
            ),
        })
    }

    pub(crate) fn err<T, T2: std::fmt::Debug>(
        &self,
        msg: &str,
        ctx: T2,
        f: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<T> {
        ftd::interpreter::utils::e2(
            format!("{}: {} ({:?}), f: {}", self.name, msg, ctx, f),
            self.name,
            line_number,
        )
    }
}

impl fastn_resolved::tdoc::TDoc for TDoc<'_> {
    fn get_opt_function(&self, name: &str) -> Option<Function> {
        match self.get_thing(name, 0).ok()? {
            ftd::interpreter::Thing::Function(r) => Some(r),
            _ => None,
        }
    }

    fn get_opt_record(&self, name: &str) -> Option<Record> {
        match self.get_thing(name, 0).ok()? {
            ftd::interpreter::Thing::Record(r) => Some(r),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        self.name
    }

    fn get_opt_component(&self, name: &str) -> Option<ComponentDefinition> {
        match self.get_thing(name, 0).ok()? {
            ftd::interpreter::Thing::Component(c) => Some(c),
            _ => None,
        }
    }

    fn get_opt_web_component(&self, name: &str) -> Option<fastn_resolved::WebComponentDefinition> {
        match self.get_thing(name, 0).ok()? {
            ftd::interpreter::Thing::WebComponent(c) => Some(c),
            _ => None,
        }
    }

    fn definitions(&self) -> &indexmap::IndexMap<String, Definition> {
        self.bag()
    }
}
