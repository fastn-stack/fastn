use ftd::interpreter::expression::ExpressionExt;

pub struct DataGenerator<'a> {
    pub doc: &'a ftd::interpreter::TDoc<'a>,
}

impl DataGenerator<'_> {
    pub(crate) fn new<'a>(doc: &'a ftd::interpreter::TDoc<'a>) -> DataGenerator<'a> {
        DataGenerator { doc }
    }

    pub(crate) fn get_data(&self) -> ftd::html::Result<ftd::Map<serde_json::Value>> {
        use ftd::interpreter::PropertyValueExt;

        let mut d: ftd::Map<serde_json::Value> = Default::default();
        for (k, v) in self.doc.bag().iter() {
            if let ftd::interpreter::Thing::Variable(fastn_resolved::Variable {
                value,
                mutable,
                line_number,
                conditional_value,
                ..
            }) = v
            {
                let mut value = value.clone();
                for conditional in conditional_value.iter() {
                    if conditional.condition.eval(self.doc)? {
                        value = conditional.value.clone();
                        break;
                    }
                }
                match value.clone().resolve(self.doc, value.line_number()) {
                    Ok(value) => {
                        if let Some(value) = ftd::interpreter::utils::get_value(self.doc, &value)? {
                            d.insert(ftd::html::utils::js_reference_name(k), value);
                        }
                    }
                    Err(e) if *mutable => {
                        return Err(ftd::html::Error::ParseError {
                            message: format!("Mutablility for inherited is not yet supported, {e}"),
                            doc_id: self.doc.name.to_string(),
                            line_number: *line_number,
                        });
                    }
                    _ => continue,
                }
            }
        }
        Ok(d)
    }
}
