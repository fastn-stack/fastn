#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: BagOrState<'a>,
}

#[derive(Debug, PartialEq)]
pub enum BagOrState<'a> {
    Bag(&'a ftd::Map<ftd::interpreter2::Thing>),
    State(&'a mut ftd::interpreter2::InterpreterState),
}

impl<'a> TDoc<'a> {
    pub fn new(
        name: &'a str,
        aliases: &'a ftd::Map<String>,
        bag: &'a ftd::Map<ftd::interpreter2::Thing>,
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
        state: &'a mut ftd::interpreter2::InterpreterState,
    ) -> TDoc<'a> {
        TDoc {
            name,
            aliases,
            bag: BagOrState::State(state),
        }
    }

    pub fn state(&'a self) -> Option<&&'a mut ftd::interpreter2::InterpreterState> {
        match &self.bag {
            BagOrState::Bag(_) => None,
            BagOrState::State(s) => Some(s),
        }
    }

    pub fn resolve_name(&self, name: &str) -> String {
        ftd::interpreter2::utils::resolve_name(name, self.name, self.aliases)
    }

    pub fn bag(&'a self) -> &'a ftd::Map<ftd::interpreter2::Thing> {
        match &self.bag {
            BagOrState::Bag(b) => b,
            BagOrState::State(s) => &s.bag,
        }
    }

    pub fn get_record(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Record> {
        match self.get_thing(name, line_number)? {
            ftd::interpreter2::Thing::Record(r) => Ok(r),
            t => self.err(
                format!("Expected Record, found: `{:?}`", t).as_str(),
                name,
                "get_record",
                line_number,
            ),
        }
    }

    pub fn search_record(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::Record>>
    {
        match self.search_thing(name, line_number)? {
            ftd::interpreter2::StateWithThing::State(s) => {
                Ok(ftd::interpreter2::StateWithThing::new_state(s))
            }
            ftd::interpreter2::StateWithThing::Thing(ftd::interpreter2::Thing::Record(r)) => {
                Ok(ftd::interpreter2::StateWithThing::new_thing(r))
            }
            ftd::interpreter2::StateWithThing::Thing(t) => self.err(
                format!("Expected Record, found: `{:?}`", t).as_str(),
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
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Variable> {
        match self.get_thing(name, line_number)? {
            ftd::interpreter2::Thing::Variable(r) => Ok(r),
            t => self.err(
                format!("Expected Variable, found: `{:?}`", t).as_str(),
                name,
                "get_variable",
                line_number,
            ),
        }
    }

    pub fn search_variable(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::Variable>>
    {
        match self.search_thing(name, line_number)? {
            ftd::interpreter2::StateWithThing::State(s) => {
                Ok(ftd::interpreter2::StateWithThing::new_state(s))
            }
            ftd::interpreter2::StateWithThing::Thing(ftd::interpreter2::Thing::Variable(r)) => {
                Ok(ftd::interpreter2::StateWithThing::new_thing(r))
            }
            ftd::interpreter2::StateWithThing::Thing(t) => self.err(
                format!("Expected Variable, found: `{:?}`", t).as_str(),
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
    ) -> ftd::interpreter2::Result<String> {
        Ok(if let Some(l) = name.strip_prefix('$') {
            let d =
                ftd::interpreter2::utils::get_doc_name_and_remaining(l, self.name, line_number).0;
            if ftd::interpreter2::utils::get_special_variable().contains(&d.as_str()) {
                return Ok(format!("${}", l));
            }
            format!("${}", self.resolve_name(l))
        } else {
            name.to_string()
        })
    }

    pub(crate) fn resolve(
        &self,
        name: &str,
        kind: &ftd::interpreter2::KindData,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
        let (value, var_name, var_line_number, remaining) =
            if let Ok(v) = self.get_initial_variable(name, line_number) {
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
                    ftd::interpreter2::PropertyValue::Value {
                        value: v.to_value(kind),
                        is_mutable: false,
                        line_number: v.line_number,
                    },
                    v.name,
                    v.line_number,
                    None,
                )
            } else {
                return ftd::interpreter2::utils::e2(
                    format!("Cannot find {} in get_thing", name),
                    self.name,
                    line_number,
                );
            };
        let value = value.resolve(self, line_number)?.inner().ok_or(
            ftd::interpreter2::Error::ParseError {
                message: format!(
                    "Expected value for variable {} in line number {}",
                    var_name, var_line_number
                ),
                doc_id: self.name.to_string(),
                line_number,
            },
        )?;
        if let Some(remaining) = remaining {
            return resolve_(remaining.as_str(), &value, line_number, self);
        }
        return Ok(value);

        fn resolve_(
            name: &str,
            value: &ftd::interpreter2::Value,
            line_number: usize,
            doc: &ftd::interpreter2::TDoc,
        ) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
            let (p1, p2) = ftd::interpreter2::utils::split_at(name, ".");
            match value {
                ftd::interpreter2::Value::Record {
                    name: rec_name,
                    fields,
                } => {
                    let field = fields
                        .get(p1.as_str())
                        .ok_or(ftd::interpreter2::Error::ParseError {
                            message: format!("Can't find field `{}` in record `{}`", p1, rec_name),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?
                        .clone()
                        .resolve(doc, line_number)?;
                    if let Some(p2) = p2 {
                        return resolve_(p2.as_str(), &field, line_number, doc);
                    }
                    Ok(field)
                }
                ftd::interpreter2::Value::List { data, kind } => {
                    let p1 = p1.parse::<usize>()?;
                    let value = data
                        .get(p1)
                        .ok_or(ftd::interpreter2::Error::ParseError {
                            message: format!(
                                "Can't find index `{}` in list of kind `{:?}`",
                                p1, kind
                            ),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?
                        .clone()
                        .resolve(doc, line_number)?;
                    if let Some(p2) = p2 {
                        return resolve_(p2.as_str(), &value, line_number, doc);
                    }
                    Ok(value)
                }
                t => ftd::interpreter2::utils::e2(
                    format!("Expected record found `{:?}`", t).as_str(),
                    doc.name,
                    line_number,
                ),
            }
        }
    }

    pub fn set_value(
        &'a self,
        name: &'a str,
        value: ftd::interpreter2::PropertyValue,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Variable> {
        let (mut variable, mut remaining) = self.get_initial_variable(name, line_number)?;

        if !variable.mutable {
            return ftd::interpreter2::utils::e2(
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
            value: &ftd::interpreter2::PropertyValue,
            name: Option<String>,
            doc: &ftd::interpreter2::TDoc,
            line_number: usize,
        ) -> ftd::interpreter2::Result<Option<(ftd::interpreter2::Variable, Option<String>)>>
        {
            let mut variable = None;
            let mut remaining = name;
            let mut value = value.clone();
            while let Some(reference) = value.reference_name() {
                let (var, rem) = doc.get_initial_variable(reference, line_number)?;
                value = var.value.clone();
                variable = Some(var);
                remaining = if let Some(remaining) = remaining {
                    Some(
                        rem.map(|v| format!("{}.{}", v, remaining))
                            .unwrap_or(remaining),
                    )
                } else {
                    rem
                };
            }

            if let ftd::interpreter2::PropertyValue::Clone { .. } = value {
                return Ok(variable.map(|v| (v, remaining)));
            }

            if let Some(ref remaining) = remaining {
                let (p1, p2) = ftd::interpreter2::utils::split_at(remaining, ".");
                let value = value.value(doc.name, line_number)?.inner().ok_or(
                    ftd::interpreter2::Error::ParseError {
                        message: format!(
                            "Value expected found null, `{:?}` in line number {}",
                            value, line_number
                        ),
                        doc_id: doc.name.to_string(),
                        line_number,
                    },
                )?;

                match value {
                    ftd::interpreter2::Value::Record {
                        name: rec_name,
                        fields,
                    } => {
                        let field_value = fields
                            .get(p1.as_str())
                            .ok_or(ftd::interpreter2::Error::ParseError {
                                message: format!(
                                    "Expected field {} in record `{}` in line number {}",
                                    p1, rec_name, line_number
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
                        return ftd::interpreter2::utils::e2(
                            format!(
                                "Expected record, found `{:?}` in line number {}",
                                t, line_number
                            )
                            .as_str(),
                            doc.name,
                            line_number,
                        )
                    }
                }
            }

            Ok(variable.map(|v| (v, remaining)))
        }

        fn set_value_(
            variable: &mut ftd::interpreter2::Variable,
            value: ftd::interpreter2::PropertyValue,
            remaining: Option<String>,
            doc: &ftd::interpreter2::TDoc,
            line_number: usize,
        ) -> ftd::interpreter2::Result<()> {
            change_value(&mut variable.value, value, remaining, doc, line_number)?;
            Ok(())
        }

        fn change_value(
            value: &mut ftd::interpreter2::PropertyValue,
            set: ftd::interpreter2::PropertyValue,
            remaining: Option<String>,
            doc: &ftd::interpreter2::TDoc,
            line_number: usize,
        ) -> ftd::interpreter2::Result<()> {
            if let Some(remaining) = remaining {
                let (p1, p2) = ftd::interpreter2::utils::split_at(remaining.as_str(), ".");
                match value {
                    ftd::interpreter2::PropertyValue::Value { value, .. } => match value {
                        ftd::interpreter2::Value::Record { name, fields } => {
                            let field = fields.get_mut(p1.as_str()).ok_or(
                                ftd::interpreter2::Error::ParseError {
                                    message: format!(
                                        "Can't find field `{}` in record `{}`",
                                        p1, name
                                    ),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                },
                            )?;
                            change_value(field, set, p2, doc, line_number)?;
                        }
                        t => {
                            return ftd::interpreter2::utils::e2(
                                format!("Expected record, found `{:?}`", t).as_str(),
                                doc.name,
                                line_number,
                            )
                        }
                    },
                    ftd::interpreter2::PropertyValue::Reference {
                        name,
                        kind,
                        is_mutable,
                        ..
                    }
                    | ftd::interpreter2::PropertyValue::Clone {
                        name,
                        kind,
                        is_mutable,
                        ..
                    } => {
                        let resolved_value = doc.resolve(name, kind, line_number)?;
                        *value = ftd::interpreter2::PropertyValue::Value {
                            value: resolved_value,
                            line_number,
                            is_mutable: *is_mutable,
                        };
                        change_value(value, set, Some(remaining), doc, line_number)?;
                    }
                    ftd::interpreter2::PropertyValue::FunctionCall(
                        ftd::interpreter2::FunctionCall {
                            name,
                            kind,
                            is_mutable,
                            values,
                            ..
                        },
                    ) => {
                        let function = doc.get_function(name, line_number)?;
                        let resolved_value = function
                            .resolve(kind, values, doc, line_number)?
                            .ok_or(ftd::interpreter2::Error::ParseError {
                                message: format!(
                                    "Expected return value of type {:?} for function {}",
                                    kind, name
                                ),
                                doc_id: doc.name.to_string(),
                                line_number,
                            })?;
                        *value = ftd::interpreter2::PropertyValue::Value {
                            value: resolved_value,
                            line_number,
                            is_mutable: *is_mutable,
                        };
                        change_value(value, set, Some(remaining), doc, line_number)?;
                    }
                }
            } else {
                assert_eq!(value.kind(), set.kind());
                *value = set;
            }
            Ok(())
        }
    }

    pub fn get_kind_with_argument(
        &'a self,
        name: &'a str,
        line_number: usize,
        component_definition_name_with_arguments: Option<(&str, &[ftd::interpreter2::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
    ) -> ftd::interpreter2::Result<
        ftd::interpreter2::StateWithThing<(
            ftd::interpreter2::PropertyValueSource,
            ftd::interpreter2::KindData,
        )>,
    > {
        let name = name
            .strip_prefix(ftd::interpreter2::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter2::utils::CLONE))
            .unwrap_or(name);

        let initial_kind_with_remaining_and_source =
            ftd::interpreter2::utils::get_argument_for_reference_and_remaining(
                name,
                self.name,
                component_definition_name_with_arguments,
                loop_object_name_and_kind,
            )
            .map(|v| (v.0.kind.to_owned(), v.1, v.2));

        let (initial_kind, remaining, source) =
            if let Some(r) = initial_kind_with_remaining_and_source {
                r
            } else {
                let (initial_thing, remaining) =
                    try_ok_state!(self.search_initial_thing(name, line_number)?);

                let initial_kind = match initial_thing {
                    ftd::interpreter2::Thing::Record(r) => {
                        ftd::interpreter2::Kind::record(r.name.as_str())
                            .into_kind_data()
                            .caption_or_body()
                    }
                    ftd::interpreter2::Thing::OrType(o) => {
                        ftd::interpreter2::Kind::or_type(o.name.as_str())
                            .into_kind_data()
                            .caption_or_body()
                    }
                    ftd::interpreter2::Thing::OrTypeWithVariant { or_type, variant } => {
                        dbg!("1.. {}, {}", &or_type, &variant.name);
                        ftd::interpreter2::Kind::or_type_with_variant(
                            or_type.as_str(),
                            variant.name.as_str(),
                        )
                        .into_kind_data()
                        .caption_or_body()
                    }
                    ftd::interpreter2::Thing::Variable(v) => v.kind,
                    ftd::interpreter2::Thing::Component(c) => {
                        ftd::interpreter2::Kind::ui_with_name(c.name.as_str())
                            .into_kind_data()
                            .caption_or_body()
                    }
                    ftd::interpreter2::Thing::Function(_) => todo!(),
                };

                (
                    initial_kind,
                    remaining,
                    ftd::interpreter2::PropertyValueSource::Global,
                )
            };

        if let Some(remaining) = remaining {
            return Ok(ftd::interpreter2::StateWithThing::new_thing((
                source,
                try_ok_state!(get_kind_(
                    initial_kind.kind,
                    remaining.as_str(),
                    self,
                    line_number
                )?),
            )));
        }

        return Ok(ftd::interpreter2::StateWithThing::new_thing((
            source,
            initial_kind,
        )));

        fn get_kind_(
            kind: ftd::interpreter2::Kind,
            name: &str,
            doc: &ftd::interpreter2::TDoc,
            line_number: usize,
        ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::KindData>>
        {
            let (v, remaining) = ftd::interpreter2::utils::split_at(name, ".");
            match kind {
                ftd::interpreter2::Kind::Record { name: rec_name } => {
                    let record = try_ok_state!(doc.search_record(rec_name.as_str(), line_number)?);
                    let field_kind = record.get_field(&v, doc.name, line_number)?.kind.to_owned();
                    if let Some(remaining) = remaining {
                        get_kind_(field_kind.kind, &remaining, doc, line_number)
                    } else {
                        Ok(ftd::interpreter2::StateWithThing::new_thing(field_kind))
                    }
                }
                ftd::interpreter2::Kind::List { kind } => {
                    if let Some(remaining) = remaining {
                        get_kind_(*kind, &remaining, doc, line_number)
                    } else {
                        Ok(ftd::interpreter2::StateWithThing::new_thing(
                            ftd::interpreter2::KindData::new(*kind),
                        ))
                    }
                }
                t => ftd::interpreter2::utils::e2(
                    format!("Expected Record field `{}`, found: `{:?}`", name, t),
                    doc.name,
                    line_number,
                ),
            }
        }
    }

    pub fn get_kind(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::KindData>>
    {
        match self.get_kind_with_argument(name, line_number, None, &None)? {
            ftd::interpreter2::StateWithThing::State(s) => {
                Ok(ftd::interpreter2::StateWithThing::new_state(s))
            }
            ftd::interpreter2::StateWithThing::Thing(fields) => {
                Ok(ftd::interpreter2::StateWithThing::new_thing(fields.1))
            }
        }
    }

    pub fn get_component(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::ComponentDefinition> {
        match self.get_thing(name, line_number)? {
            ftd::interpreter2::Thing::Component(c) => Ok(c),
            t => self.err(
                format!("Expected Component, found: `{:?}`", t).as_str(),
                name,
                "get_component",
                line_number,
            ),
        }
    }

    pub fn search_component(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<
        ftd::interpreter2::StateWithThing<ftd::interpreter2::ComponentDefinition>,
    > {
        match self.search_thing(name, line_number)? {
            ftd::interpreter2::StateWithThing::State(s) => {
                Ok(ftd::interpreter2::StateWithThing::new_state(s))
            }
            ftd::interpreter2::StateWithThing::Thing(ftd::interpreter2::Thing::Component(c)) => {
                Ok(ftd::interpreter2::StateWithThing::new_thing(c))
            }
            ftd::interpreter2::StateWithThing::Thing(t) => self.err(
                format!("Expected Component, found: `{:?}`", t).as_str(),
                name,
                "search_component",
                line_number,
            ),
        }
    }

    pub fn search_or_type(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::OrType>>
    {
        match self.search_thing(name, line_number)? {
            ftd::interpreter2::StateWithThing::State(s) => {
                Ok(ftd::interpreter2::StateWithThing::new_state(s))
            }
            ftd::interpreter2::StateWithThing::Thing(ftd::interpreter2::Thing::OrType(c)) => {
                Ok(ftd::interpreter2::StateWithThing::new_thing(c))
            }
            ftd::interpreter2::StateWithThing::Thing(t) => self.err(
                format!("Expected Component, found: `{:?}`", t).as_str(),
                name,
                "search_component",
                line_number,
            ),
        }
    }

    pub fn get_thing(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Thing> {
        let name = name
            .strip_prefix(ftd::interpreter2::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter2::utils::CLONE))
            .unwrap_or(name);

        let (initial_thing, remaining) = self.get_initial_thing(name, line_number)?;

        if let Some(remaining) = remaining {
            return get_thing_(self, line_number, remaining.as_str(), &initial_thing);
        }
        return Ok(initial_thing);

        fn get_thing_(
            doc: &ftd::interpreter2::TDoc,
            line_number: usize,
            name: &str,
            thing: &ftd::interpreter2::Thing,
        ) -> ftd::interpreter2::Result<ftd::interpreter2::Thing> {
            let (v, remaining) = ftd::interpreter2::utils::split_at(name, ".");
            let thing = match thing.clone() {
                ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                    name,
                    value,
                    mutable,
                    ..
                }) => {
                    let value_kind = value.kind();
                    let fields = match value.resolve(doc, line_number)?.inner() {
                        Some(ftd::interpreter2::Value::Record { fields, .. }) => fields,
                        Some(ftd::interpreter2::Value::Object { values }) => values,
                        Some(ftd::interpreter2::Value::List { data, .. }) => data
                            .into_iter()
                            .enumerate()
                            .map(|(index, v)| (index.to_string(), v))
                            .collect::<ftd::Map<ftd::interpreter2::PropertyValue>>(),
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
                                ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                                    name,
                                    kind: kind.to_owned(),
                                    mutable,
                                    value: ftd::interpreter2::PropertyValue::Value {
                                        value: ftd::interpreter2::Value::Optional {
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
                                return get_thing_(doc, line_number, &remaining, &thing);
                            }
                            return Ok(thing);
                        }
                        _ => return doc.err("not an record", thing, "get_thing", line_number),
                    };
                    match fields.get(&v) {
                        Some(ftd::interpreter2::PropertyValue::Value {
                            value: val,
                            line_number,
                            is_mutable,
                        }) => ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                            name,
                            kind: ftd::interpreter2::KindData {
                                kind: val.kind(),
                                caption: false,
                                body: false,
                            },
                            mutable: false,
                            value: ftd::interpreter2::PropertyValue::Value {
                                value: val.to_owned(),
                                line_number: *line_number,
                                is_mutable: *is_mutable,
                            },
                            conditional_value: vec![],
                            line_number: *line_number,
                            is_static: !mutable,
                        }),
                        Some(ftd::interpreter2::PropertyValue::Reference { name, .. })
                        | Some(ftd::interpreter2::PropertyValue::Clone { name, .. }) => {
                            let (initial_thing, name) = doc.get_initial_thing(name, line_number)?;
                            if let Some(remaining) = name {
                                get_thing_(doc, line_number, remaining.as_str(), &initial_thing)?
                            } else {
                                initial_thing
                            }
                        }
                        _ => thing.clone(),
                    }
                }
                _ => {
                    return doc.err("not an or-type", thing, "get_thing", line_number);
                }
            };
            if let Some(remaining) = remaining {
                return get_thing_(doc, line_number, &remaining, &thing);
            }
            Ok(thing)
        }
    }

    pub fn get_function(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Function> {
        let initial_thing = self.get_initial_thing(name, line_number)?.0;
        initial_thing.function(self.name, line_number)
    }

    pub fn search_function(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::Function>>
    {
        let initial_thing = try_ok_state!(self.search_initial_thing(name, line_number)?).0;
        Ok(ftd::interpreter2::StateWithThing::new_thing(
            initial_thing.function(self.name, line_number)?,
        ))
    }

    pub fn get_initial_variable(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<(ftd::interpreter2::Variable, Option<String>)> {
        let (initial_thing, remaining) = self.get_initial_thing(name, line_number)?;
        Ok((initial_thing.variable(self.name, line_number)?, remaining))
    }

    pub fn search_thing(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::Thing>>
    {
        let name = dbg!(name
            .strip_prefix(ftd::interpreter2::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter2::utils::CLONE))
            .unwrap_or(name));

        let (initial_thing, remaining) =
            try_ok_state!(self.search_initial_thing(name, line_number)?);

        if let Some(remaining) = remaining {
            return dbg!(search_thing_(
                self,
                line_number,
                remaining.as_str(),
                initial_thing
            ));
        }
        return Ok(ftd::interpreter2::StateWithThing::new_thing(initial_thing));

        fn search_thing_(
            doc: &ftd::interpreter2::TDoc,
            line_number: usize,
            name: &str,
            thing: ftd::interpreter2::Thing,
        ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::Thing>>
        {
            let (v, remaining) = ftd::interpreter2::utils::split_at(name, ".");
            let thing = match thing.clone() {
                ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                    name,
                    value,
                    mutable,
                    ..
                }) => {
                    let value_kind = value.kind();
                    let fields = match value.resolve(doc, line_number)?.inner() {
                        Some(ftd::interpreter2::Value::Record { fields, .. }) => fields,
                        Some(ftd::interpreter2::Value::Object { values }) => values,
                        Some(ftd::interpreter2::Value::List { data, .. }) => data
                            .into_iter()
                            .enumerate()
                            .map(|(index, v)| (index.to_string(), v))
                            .collect::<ftd::Map<ftd::interpreter2::PropertyValue>>(),
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
                                ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                                    name,
                                    kind: kind.to_owned(),
                                    mutable,
                                    value: ftd::interpreter2::PropertyValue::Value {
                                        value: ftd::interpreter2::Value::Optional {
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
                            return Ok(ftd::interpreter2::StateWithThing::new_thing(thing));
                        }
                        _ => return doc.err("not an record", thing, "search_thing_", line_number),
                    };
                    match fields.get(&v) {
                        Some(ftd::interpreter2::PropertyValue::Value {
                            value: val,
                            line_number,
                            is_mutable,
                        }) => ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                            name,
                            kind: ftd::interpreter2::KindData {
                                kind: val.kind(),
                                caption: false,
                                body: false,
                            },
                            mutable: false,
                            value: ftd::interpreter2::PropertyValue::Value {
                                value: val.to_owned(),
                                line_number: *line_number,
                                is_mutable: *is_mutable,
                            },
                            conditional_value: vec![],
                            line_number: *line_number,
                            is_static: !mutable,
                        }),
                        Some(ftd::interpreter2::PropertyValue::Reference { name, .. })
                        | Some(ftd::interpreter2::PropertyValue::Clone { name, .. }) => {
                            let (initial_thing, name) =
                                try_ok_state!(doc.search_initial_thing(name, line_number)?);

                            if let Some(remaining) = name {
                                try_ok_state!(search_thing_(
                                    doc,
                                    line_number,
                                    remaining.as_str(),
                                    initial_thing,
                                )?)
                            } else {
                                initial_thing
                            }
                        }
                        _ => thing,
                    }
                }
                ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                    name: or_type_name,
                    variants,
                    ..
                }) => {
                    if let Some(thing) = variants.into_iter().find(|v| {
                        v.name
                            .trim_start_matches(format!("{}.", or_type_name).as_str())
                            .eq(name)
                    }) {
                        ftd::interpreter2::Thing::OrTypeWithVariant {
                            or_type: or_type_name.to_string(),
                            variant: thing,
                        }
                    } else {
                        return doc.err(
                            format!(
                                "Can't find variant `{}` in or-type `{}`",
                                name, or_type_name
                            )
                            .as_str(),
                            thing,
                            "search_thing_",
                            line_number,
                        );
                    }
                }
                _ => {
                    return doc.err(
                        format!("not an or-type `{}`", name).as_str(),
                        thing,
                        "search_thing_",
                        line_number,
                    );
                }
            };
            if let Some(remaining) = remaining {
                return search_thing_(doc, line_number, &remaining, thing);
            }
            Ok(ftd::interpreter2::StateWithThing::new_thing(thing))
        }
    }

    pub fn search_initial_thing(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<
        ftd::interpreter2::StateWithThing<(ftd::interpreter2::Thing, Option<String>)>,
    > {
        use itertools::Itertools;

        let name = name
            .strip_prefix(ftd::interpreter2::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter2::utils::CLONE))
            .unwrap_or(name);

        if let Ok(thing) = self.get_initial_thing(name, line_number) {
            return Ok(ftd::interpreter2::StateWithThing::new_thing(thing));
        }

        let mut state = if let Some(state) = self.state() {
            (*state).clone()
        } else {
            return self.err("not found", name, "search_thing", line_number);
        };

        let name = self.resolve_name(name);

        let (doc_name, thing_name, remaining) = // Todo: use remaining
            ftd::interpreter2::utils::get_doc_name_and_thing_name_and_remaining(
                name.as_str(),
                self.name,
                line_number,
            );

        let current_parsed_document = state.parsed_libs.get(self.name).unwrap();

        if doc_name.ne(self.name) {
            let current_doc_contains_thing = current_parsed_document
                .ast
                .iter()
                .filter(|v| {
                    !v.is_component()
                        && (v.name().eq(&format!("{}.{}", doc_name, thing_name))
                            || v.name()
                                .starts_with(format!("{}.{}.", doc_name, thing_name).as_str()))
                })
                .map(|v| v.to_owned())
                .collect_vec();
            if !current_doc_contains_thing.is_empty() {
                state
                    .to_process
                    .push((self.name.to_string(), current_doc_contains_thing));
            }
        }

        if let Some(parsed_document) = state.parsed_libs.get(doc_name.as_str()) {
            let ast_for_thing = parsed_document
                .ast
                .iter()
                .filter(|v| {
                    !v.is_component()
                        && (v.name().eq(&thing_name)
                            || v.name().starts_with(format!("{}.", thing_name).as_str()))
                })
                .map(|v| v.to_owned())
                .collect_vec();

            if ast_for_thing.is_empty() {
                if parsed_document
                    .foreign_variable
                    .iter()
                    .any(|v| thing_name.eq(v))
                {
                    return Ok(ftd::interpreter2::StateWithThing::new_state(
                        ftd::interpreter2::Interpreter::StuckOnForeignVariable {
                            module: doc_name,
                            state,
                            variable: remaining
                                .map(|v| format!("{}.{}", thing_name, v))
                                .unwrap_or(thing_name),
                        },
                    ));
                }

                return self.err("not found", name, "search_thing", line_number);
            }

            state.to_process.push((doc_name, ast_for_thing));

            return Ok(ftd::interpreter2::StateWithThing::new_state(
                state.continue_()?,
            ));
        }

        if doc_name.eq(self.name) {
            return self.err("not found", name, "search_thing", line_number);
        }

        Ok(ftd::interpreter2::StateWithThing::new_state(
            ftd::interpreter2::Interpreter::StuckOnImport {
                module: doc_name,
                state,
            },
        ))
    }

    pub fn get_initial_thing(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<(ftd::interpreter2::Thing, Option<String>)> {
        let name = name
            .strip_prefix(ftd::interpreter2::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter2::utils::CLONE))
            .unwrap_or(name);

        if name.contains('#') {
            let (splited_name, remaining_value) = if let Ok(function_name) =
                ftd::interpreter2::utils::get_function_name(name, self.name, line_number)
            {
                (function_name, None)
            } else {
                ftd::interpreter2::utils::get_doc_name_and_remaining(name, self.name, line_number)
            };
            return match self.bag().get(name) {
                Some(a) => Ok((a.to_owned(), None)),
                None => match self.bag().get(splited_name.as_str()) {
                    Some(a) => Ok((a.to_owned(), remaining_value)),
                    None => self.err("not found", splited_name, "get_initial_thing", line_number),
                },
            };
        }

        return Ok(
            match get_initial_thing_(self, self.name, name, line_number) {
                Some(a) => a,
                None => {
                    if let Some((m, v)) = name.split_once('.') {
                        match get_initial_thing_(self, m, v, line_number) {
                            None => {
                                return self.err(
                                    "not found",
                                    name,
                                    "get_initial_thing",
                                    line_number,
                                )
                            }
                            Some(a) => a,
                        }
                    } else {
                        return self.err("not found", name, "get_initial_thing", line_number);
                    }
                }
            },
        );

        fn get_initial_thing_(
            doc: &ftd::interpreter2::TDoc,
            doc_name: &str,
            name: &str,
            line_number: usize,
        ) -> Option<(ftd::interpreter2::Thing, Option<String>)> {
            match doc
                .bag()
                .get(format!("{}#{}", doc_name, name).as_str())
                .map(ToOwned::to_owned)
            {
                Some(a) => return Some((a, None)),
                None => {
                    if let Some(g) = doc.aliases.get(doc_name) {
                        if let Some(a) = doc
                            .bag()
                            .get(format!("{}#{}", g, name).as_str())
                            .map(|v| (v.clone(), None))
                        {
                            return Some(a);
                        }
                    }
                }
            }

            let (splited_name, remaining_value) = if let Ok(function_name) =
                ftd::interpreter2::utils::get_function_name(name, doc.name, line_number)
            {
                (function_name, None)
            } else if let Some((v, remaining_value)) = name.split_once('.') {
                (v.to_string(), Some(remaining_value.to_string()))
            } else {
                (name.to_string(), None)
            };

            match doc
                .bag()
                .get(format!("{}#{}", doc_name, splited_name).as_str())
                .map(ToOwned::to_owned)
            {
                Some(a) => Some((a, remaining_value)),
                None => match doc.aliases.get(doc_name) {
                    Some(g) => doc
                        .bag()
                        .get(format!("{}#{}", g, splited_name).as_str())
                        .map(|v| (v.clone(), remaining_value)),
                    None => None,
                },
            }
        }
    }

    pub(crate) fn err<T, T2: std::fmt::Debug>(
        &self,
        msg: &str,
        ctx: T2,
        f: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<T> {
        ftd::interpreter2::utils::e2(
            format!("{}: {} ({:?}), f: {}", self.name, msg, ctx, f),
            self.name,
            line_number,
        )
    }
}
