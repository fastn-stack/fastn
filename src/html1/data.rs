pub struct DataGenerator<'a> {
    pub doc: &'a ftd::interpreter2::TDoc<'a>,
}

impl<'a> DataGenerator<'a> {
    pub(crate) fn new(doc: &'a ftd::interpreter2::TDoc<'a>) -> DataGenerator<'a> {
        DataGenerator { doc }
    }

    pub(crate) fn get_data(&self) -> ftd::html1::Result<ftd::Map<serde_json::Value>> {
        let mut d: ftd::Map<serde_json::Value> = Default::default();
        for (k, v) in self.doc.bag().iter() {
            if let ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                value, ..
            }) = v
            {
                let value = value.clone().resolve(self.doc, value.line_number())?;
                if let Some(value) = ftd::interpreter2::utils::get_value(self.doc, &value)? {
                    d.insert(ftd::html1::utils::js_reference_name(k), value);
                }
            }
        }
        Ok(d)
    }
}
