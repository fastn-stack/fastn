pub use ftd::p1::{Error, Result};

#[derive(Debug, PartialEq, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Header(pub Vec<(usize, String, String)>);

#[derive(PartialEq)]
pub enum CheckType {
    Component,
    Variable,
}

impl Header {
    pub fn component_dup_header_check(
        &self,
        id: &str,
        doc: &ftd::p2::TDoc,
        sub_sections: Option<&ftd::p1::SubSections>,
        var_types: &[String],
    ) -> ftd::p1::Result<()> {
        // id = file_name, name = section name
        let mut header_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (ln, key, _) in self.0.iter() {
            // Ignore commented headers and lines starting with << or >>
            // Ignore processor keywords
            if key.starts_with('/')
                || key.starts_with('>')
                || key.starts_with('<')
                || (key.starts_with('$') && key.ends_with('$'))
            {
                continue;
            }

            // [Key_Type] [Identifier] = Header Key (eg. boolean flag)
            // No Key_Type during invocation

            // key_tokens = [ <Identfier> , [Key_Type] ]
            let key_tokens: Vec<&str> = key.as_str().rsplit(' ').collect();
            let identifier = key_tokens[0];

            // If header found again throw error
            if header_set.contains(identifier) {
                return Err(ftd::p1::Error::ParseError {
                    message: format!("Header {} is already defined ", identifier),
                    doc_id: id.to_string(),
                    line_number: *ln,
                });
            }
            // Else insert it in the header set
            header_set.insert(identifier.to_string());
        }

        // If mode is section then check its subsections if available
        // since sub-sections dont have any further nesting i.e no sub-sub-sections
        if sub_sections != None {
            self.check_sub_sections(id, sub_sections, doc, var_types, None, CheckType::Component)?;
        }

        Ok(())
    }

    pub fn var_dup_header_check(
        &self,
        id: &str,
        var_data: &ftd::variable::VariableData,
        doc: &ftd::p2::TDoc,
        p1_line_number: usize,
        sub_sections: Option<&ftd::p1::SubSections>,
        var_types: &[String],
    ) -> ftd::p1::Result<()> {
        // VariableData attributes
        let bag = doc.bag;
        let kind = var_data.kind.to_string();

        // Ignoring those kinds whose bag entry is not there
        if kind.eq("string") || kind.eq("object") || kind.eq("integer") {
            return Ok(());
        }

        let mut header_set: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Bag = Map( String -> Thing )
        // General Key Entry in bag = [file_name/id]#[kind]
        let _root = doc.resolve_name(p1_line_number, var_data.kind.as_str())?;

        let bag_entry = {
            // For index.ftd bag entry format = ftd#[kind ignoring the 'ftd.' part]
            // For foo/bar (used in test units), bag entry can be
            // ftd#[kind ignoring the 'ftd.' part] if the kind is a std kind
            // or
            // foo/bar#[kind only including the parent kind] if the kind is not std kind
            // For other files bag entry = [file_name/id]#[kind]
            if id.eq("index.ftd") {
                let tokens: Vec<&str> = kind.split('.').collect();
                format!("ftd#{}", tokens[tokens.len() - 1])
            } else if id.eq("foo/bar") {
                let tokens: Vec<&str> = kind.split('.').collect();
                let std_type_key = format!("ftd#{}", tokens[tokens.len() - 1]);
                if bag.contains_key(&std_type_key) {
                    std_type_key
                } else {
                    format!("{}#{}", id, tokens[0])
                }
            } else {
                format!("{}#{}", id, kind)
            }
        };

        if bag.contains_key(&bag_entry) {
            // Check if the thing (section) is record then evaluate its headers for duplicates
            let thing: &ftd::p2::Thing = &bag[&bag_entry];
            // println!("thing = {:?}", thing);

            // Currently handling only for record variables
            if let ftd::p2::Thing::Record(ref rec) = thing {
                let header_list = self;
                for (ln, key, _) in header_list.0.iter() {
                    // Ignore commented headers and lines starting with << or >>
                    // Ignore processor keywords
                    if key.starts_with('/')
                        || key.starts_with('>')
                        || key.starts_with('<')
                        || (key.starts_with('$') && key.ends_with('$'))
                    {
                        continue;
                    }

                    // Record.fields: Map( String -> Kind ) or Map( Headers -> Kind )
                    // Allow repeated use of list inside record variables
                    if rec.fields.contains_key(key) {
                        if rec.fields[key].is_list() {
                            continue;
                        }
                    } else {
                        return Err(ftd::p1::Error::ParseError {
                            message: format!("Invalid header '{}' not found !!", key),
                            doc_id: id.to_string(),
                            line_number: *ln,
                        });
                    }

                    // Otherwise function normally for other headers
                    if header_set.contains(key) {
                        return Err(ftd::p1::Error::ParseError {
                            message: format!(
                                "repeated use of header '{}' in record {} not allowed !!",
                                key, kind
                            ),
                            doc_id: id.to_string(),
                            line_number: *ln,
                        });
                    }
                    header_set.insert(key.to_string());
                }

                if sub_sections != None {
                    self.check_sub_sections(
                        id,
                        sub_sections,
                        doc,
                        var_types,
                        Some(&rec.fields),
                        CheckType::Variable,
                    )?;
                }
            }
        } else {
            // Bag Entry not found
            return Err(ftd::p1::Error::NotFound {
                key: bag_entry,
                doc_id: id.to_string(),
                line_number: p1_line_number,
            });
        }

        Ok(())
    }

    pub fn var_dup_header_check_sub_section(
        &self,
        id: &str,
        name: &str,
        bag: &std::collections::BTreeMap<String, ftd::p2::Thing>,
        p1_line_number: usize,
        fields: Option<&std::collections::BTreeMap<String, ftd::p2::Kind>>,
    ) -> ftd::p1::Result<()> {
        let mut header_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        if let Some(f) = fields {

            // header.x1.x2 ....
            let name_tokens: Vec<&str> = name.split('.').collect();
            let header_key = name_tokens[0];

            if f.contains_key(header_key) {
                // Determine the kind of the sub-section (inside var)
                let kind = &f[header_key];

                match kind {
                    ftd::p2::Kind::Record { name, .. } => {
                        // Determine the record name entry for the sub-section (inside var)
                        let bag_entry = name;
                        let thing: &ftd::p2::Thing = &bag[bag_entry];

                        let header_list = &self.0;

                        if let ftd::p2::Thing::Record(ref rec) = thing {
                            for (ln, key, _val) in header_list.iter() {
                                // Ignore commented headers and lines starting with << or >>
                                // Ignore processor keywords
                                if key.starts_with('/')
                                    || key.starts_with('>')
                                    || key.starts_with('<')
                                    || (key.starts_with('$') && key.ends_with('$'))
                                {
                                    continue;
                                }

                                // Check if the header is valid and if valid ignore its
                                // dups if the header is list type
                                if rec.fields.contains_key(key) {
                                    if rec.fields[key].is_list() {
                                        continue;
                                    }
                                } else {
                                    return Err(ftd::p1::Error::ParseError {
                                        message: format!("Invalid header '{}' not found !!", key),
                                        doc_id: id.to_string(),
                                        line_number: *ln,
                                    });
                                }

                                // Otherwise function normally for other headers
                                if header_set.contains(key) {
                                    return Err(ftd::p1::Error::ParseError {
                                        message: format!(
                                            "repeated use of header '{}' not allowed !!",
                                            key
                                        ),
                                        doc_id: id.to_string(),
                                        line_number: *ln,
                                    });
                                }
                                header_set.insert(key.to_string());
                            }
                        }
                    }
                    ftd::p2::Kind::OrType { .. } => {
                        // Need to work on this
                    }
                    _ => {
                        // TODO: Case for list of records
                        // TODO: Or_type
                        // No other cases handled for now
                    }
                }
            } else {
                return Err(ftd::p1::Error::ParseError {
                    message: format!("{} is not a valid sub_section !!", header_key),
                    doc_id: id.to_string(),
                    line_number: p1_line_number,
                });
            }
        }

        Ok(())
    }

    pub fn check_sub_sections(
        &self,
        id: &str,
        sub_sections: Option<&ftd::p1::SubSections>,
        doc: &ftd::p2::TDoc,
        var_types: &[String],
        fields: Option<&std::collections::BTreeMap<String, ftd::p2::Kind>>,
        check_type: CheckType,
    ) -> ftd::p1::Result<()> {
        let bag = doc.bag;
        // Make header checks for rest of the subsections if available
        if let Some(sub_sections) = sub_sections {
            let sub_sections_list = &sub_sections.0;
            for sub in sub_sections_list {
                let sub_name = &sub.name;
                let sub_var_data = ftd::variable::VariableData::get_name_kind(
                    sub_name,
                    doc,
                    sub.line_number,
                    var_types,
                );

                // Separate checks for different sub-section types
                if sub_name.starts_with("record ") {
                    // For record declaration
                    if let Ok(ref _s) = sub_var_data {
                        sub.header
                            .component_dup_header_check(id, doc, None, var_types)?;
                    }
                } else if sub_name.starts_with("or-type ")
                    || sub_name.starts_with("map ")
                    || sub_name == "container"
                {
                    // No checks for now
                } else if let Ok(ftd::variable::VariableData {
                    type_: ftd::variable::Type::Component,
                    ..
                }) = sub_var_data
                {
                    // For variable component
                    if let Ok(ref _s) = sub_var_data {
                        sub.header
                            .component_dup_header_check(id, doc, None, var_types)?;
                    }
                } else if let Ok(ref sub_var_data) = sub_var_data {
                    if sub_var_data.is_none() || sub_var_data.is_optional() {
                        // For variables
                        sub.header.var_dup_header_check_sub_section(
                            id,
                            sub_name,
                            bag,
                            sub.line_number,
                            fields,
                        )?;
                    }
                } else {
                    // For invocation
                    if check_type == CheckType::Component {
                        sub.header
                            .component_dup_header_check(id, doc, None, var_types)?;
                    } else if check_type == CheckType::Variable {
                        // Sub-section is invoked inside variable on the defined parameters
                        sub.header.var_dup_header_check_sub_section(
                            id,
                            sub_name,
                            bag,
                            sub.line_number,
                            fields,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn without_line_number(&self) -> Self {
        let mut header: Header = Default::default();
        for (_, k, v) in self.0.iter() {
            header.add(&0, k, v);
        }
        header
    }

    pub fn add(&mut self, line_number: &usize, name: &str, value: &str) {
        self.0
            .push((*line_number, name.to_string(), value.to_string()))
    }

    pub fn bool_with_default(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
        def: bool,
    ) -> Result<bool> {
        match self.bool(doc_id, line_number, name) {
            Ok(b) => Ok(b),
            Err(Error::NotFound { .. }) => Ok(def),
            e => e,
        }
    }

    pub fn bool_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> Result<Option<bool>> {
        match self.bool(doc_id, line_number, name) {
            Ok(b) => Ok(Some(b)),
            Err(Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn bool(&self, doc_id: &str, line_number: usize, name: &str) -> Result<bool> {
        for (l, k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }
            if k == name {
                return if v == "true" || v == "false" {
                    Ok(v == "true")
                } else {
                    Err(ftd::p1::Error::ParseError {
                        message: "can't parse bool".to_string(),
                        doc_id: doc_id.to_string(),
                        line_number: *l,
                    })
                };
            }
        }
        Err(Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: name.to_string(),
        })
    }

    pub fn i32_with_default(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
        def: i32,
    ) -> Result<i32> {
        match self.i32(doc_id, line_number, name) {
            Ok(b) => Ok(b),
            Err(Error::NotFound { .. }) => Ok(def),
            e => e,
        }
    }

    pub fn i32_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> Result<Option<i32>> {
        match self.i32(doc_id, line_number, name) {
            Ok(b) => Ok(Some(b)),
            Err(Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn i32(&self, doc_id: &str, line_number: usize, name: &str) -> Result<i32> {
        for (l, k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }
            if k == name {
                return v.parse().map_err(|e: std::num::ParseIntError| {
                    ftd::p1::Error::ParseError {
                        message: format!("{:?}", e),
                        doc_id: doc_id.to_string(),
                        line_number: *l,
                    }
                });
            }
        }
        Err(Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: name.to_string(),
        })
    }

    pub fn i64(&self, doc_id: &str, line_number: usize, name: &str) -> Result<i64> {
        for (l, k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }

            if k == name {
                return v.parse().map_err(|e: std::num::ParseIntError| {
                    ftd::p1::Error::ParseError {
                        message: format!("{:?}", e),
                        doc_id: doc_id.to_string(),
                        line_number: *l,
                    }
                });
            }
        }
        Err(Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: name.to_string(),
        })
    }

    pub fn i64_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> Result<Option<i64>> {
        match self.i64(doc_id, line_number, name) {
            Ok(b) => Ok(Some(b)),
            Err(Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn f64(&self, doc_id: &str, line_number: usize, name: &str) -> Result<f64> {
        for (l, k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }

            if k == name {
                return v.parse().map_err(|e: std::num::ParseFloatError| {
                    ftd::p1::Error::ParseError {
                        message: format!("{:?}", e),
                        doc_id: doc_id.to_string(),
                        line_number: *l,
                    }
                });
            }
        }
        Err(Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: name.to_string(),
        })
    }

    pub fn f64_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> Result<Option<f64>> {
        match self.f64(doc_id, line_number, name) {
            Ok(b) => Ok(Some(b)),
            Err(Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn str_with_default<'a>(
        &'a self,
        doc_id: &str,
        line_number: usize,
        name: &str,
        def: &'a str,
    ) -> Result<&'a str> {
        match self.str(doc_id, line_number, name) {
            Ok(b) => Ok(b),
            Err(Error::NotFound { .. }) => Ok(def),
            e => e,
        }
    }

    pub fn get_events(
        &self,
        line_number: usize,
        doc: &ftd::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    ) -> ftd::p1::Result<Vec<ftd::p2::Event>> {
        let events = {
            let mut events = vec![];
            for (_, k, v) in self.0.iter() {
                if k.starts_with("$on-") && k.ends_with('$') {
                    let mut event = k.replace("$on-", "");
                    event = event[..event.len() - 1].to_string();
                    events.push((event, v.to_string()));
                }
            }
            events
        };
        let mut event = vec![];
        for (e, a) in events {
            event.push(ftd::p2::Event::to_event(
                line_number,
                &e,
                &a,
                doc,
                arguments,
            )?);
        }
        Ok(event)
    }

    pub fn str_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> Result<Option<&str>> {
        match self.str(doc_id, line_number, name) {
            Ok(b) => Ok(Some(b)),
            Err(Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn conditional_str(
        &self,
        doc: &ftd::p2::TDoc,
        line_number: usize,
        name: &str,
        arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    ) -> Result<Vec<(usize, String, Option<&str>)>> {
        let mut conditional_vector = vec![];
        for (idx, (_, k, v)) in self.0.iter().enumerate() {
            let v = doc.resolve_reference_name(line_number, v, arguments)?;
            if k == name {
                conditional_vector.push((idx, v.to_string(), None));
            }
            if k.contains(" if ") {
                let mut parts = k.splitn(2, " if ");
                let property_name = parts.next().unwrap().trim();
                if property_name == name {
                    let conditional_attribute = parts.next().unwrap().trim();
                    conditional_vector.push((idx, v.to_string(), Some(conditional_attribute)));
                }
            }
        }
        if conditional_vector.is_empty() {
            Err(Error::NotFound {
                doc_id: doc.name.to_string(),
                line_number,
                key: format!("`{}` header is missing", name),
            })
        } else {
            Ok(conditional_vector)
        }
    }

    pub fn str(&self, doc_id: &str, line_number: usize, name: &str) -> Result<&str> {
        for (_, k, v) in self.0.iter() {
            if k.starts_with('/') {
                continue;
            }
            if k == name {
                return Ok(v.as_str());
            }
        }

        Err(Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: format!("`{}` header is missing", name),
        })
    }

    pub fn string(&self, doc_id: &str, line_number: usize, name: &str) -> Result<String> {
        self.str(doc_id, line_number, name).map(ToString::to_string)
    }

    pub fn string_optional(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> Result<Option<String>> {
        Ok(self
            .str_optional(doc_id, line_number, name)?
            .map(ToString::to_string))
    }

    pub fn string_with_default(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
        def: &str,
    ) -> Result<String> {
        self.str_with_default(doc_id, line_number, name, def)
            .map(ToString::to_string)
    }
}
