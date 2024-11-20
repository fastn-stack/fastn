#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a Map<String>,
    pub bag: &'a indexmap::IndexMap<String, fastn_resolved::Definition>,
}

type Map<T> = std::collections::BTreeMap<String, T>;
type Result<T> = std::result::Result<T, String>;

pub const REFERENCE: &str = "$";
pub const CLONE: &str = "*$";

impl<'a> TDoc<'a> {
    pub(crate) fn err<T, T2: std::fmt::Debug>(
        &self,
        _msg: &str,
        _ctx: T2,
        _f: &str,
        _line_number: usize,
    ) -> Result<T> {
        todo!()
        // ftd::interpreter::utils::e2(
        //     format!("{}: {} ({:?}), f: {}", self.name, msg, ctx, f),
        //     self.name,
        //     line_number,
        // )
    }

    pub fn get_record(
        &'a self,
        _name: &'a str,
        _line_number: usize,
    ) -> Result<fastn_resolved::Record> {
        todo!()
        // match self.get_thing(name, line_number)? {
        //     fastn_resolved::Definition::Record(r) => Ok(r),
        //     t => self.err(
        //         format!("Expected Record, found: `{:?}`", t).as_str(),
        //         name,
        //         "get_record",
        //         line_number,
        //     ),
        // }
    }

    // pub fn get_thing(
    //     &'a self,
    //     name: &'a str,
    //     line_number: usize,
    // ) -> Result<fastn_resolved::Definition> {
    //     let name = name
    //         .strip_prefix(REFERENCE)
    //         .or_else(|| name.strip_prefix(CLONE))
    //         .unwrap_or(name);
    //
    //     let (initial_thing, remaining) = self.get_initial_thing(name, line_number)?;
    //
    //     if let Some(remaining) = remaining {
    //         return get_thing_(self, line_number, remaining.as_str(), &initial_thing);
    //     }
    //     return Ok(initial_thing);
    //
    //     fn get_thing_(
    //         doc: &TDoc,
    //         line_number: usize,
    //         name: &str,
    //         thing: &fastn_resolved::Definition,
    //     ) -> Result<fastn_resolved::Definition> {
    //         use ftd::interpreter::PropertyValueExt;
    //         use itertools::Itertools;
    //
    //         let (v, remaining) = ftd::interpreter::utils::split_at(name, ".");
    //         let thing = match thing.clone() {
    //             fastn_resolved::Definition::Variable(fastn_resolved::Variable {
    //                 name,
    //                 value,
    //                 mutable,
    //                 ..
    //             }) => {
    //                 let value_kind = value.kind();
    //                 let fields = match value.resolve(doc, line_number)?.inner() {
    //                     Some(fastn_resolved::Value::Record { fields, .. }) => fields,
    //                     Some(fastn_resolved::Value::Object { values }) => values,
    //                     Some(fastn_resolved::Value::KwArgs { arguments }) => arguments,
    //                     Some(fastn_resolved::Value::List { data, .. }) => data
    //                         .into_iter()
    //                         .enumerate()
    //                         .map(|(index, v)| (index.to_string(), v))
    //                         .collect::<Map<fastn_resolved::PropertyValue>>(),
    //                     None => {
    //                         let kind_name = match value_kind.get_record_name() {
    //                             Some(name) => name,
    //                             _ => {
    //                                 return doc.err(
    //                                     "not an record",
    //                                     thing,
    //                                     "get_thing",
    //                                     line_number,
    //                                 );
    //                             }
    //                         };
    //                         let kind_thing = doc.get_thing(kind_name, line_number)?;
    //                         let kind = match kind_thing
    //                             .record(doc.name, line_number)?
    //                             .fields
    //                             .iter()
    //                             .find(|f| f.name.eq(&v))
    //                             .map(|v| v.kind.to_owned())
    //                         {
    //                             Some(f) => f,
    //                             _ => {
    //                                 return doc.err(
    //                                     "not an record or or-type",
    //                                     thing,
    //                                     "get_thing",
    //                                     line_number,
    //                                 );
    //                             }
    //                         };
    //                         let thing =
    //                             ftd::interpreter::Thing::Variable(fastn_resolved::Variable {
    //                                 name,
    //                                 kind: kind.to_owned(),
    //                                 mutable,
    //                                 value: fastn_resolved::PropertyValue::Value {
    //                                     value: fastn_resolved::Value::Optional {
    //                                         data: Box::new(None),
    //                                         kind,
    //                                     },
    //                                     is_mutable: mutable,
    //                                     line_number,
    //                                 },
    //                                 conditional_value: vec![],
    //                                 line_number,
    //                                 is_static: !mutable,
    //                             });
    //                         if let Some(remaining) = remaining {
    //                             return get_thing_(doc, line_number, &remaining, &thing);
    //                         }
    //                         return Ok(thing);
    //                     }
    //                     _ => return doc.err("not an record", thing, "get_thing", line_number),
    //                 };
    //                 match fields.get(&v) {
    //                     Some(fastn_resolved::PropertyValue::Value {
    //                         value: val,
    //                         line_number,
    //                         is_mutable,
    //                     }) => ftd::interpreter::Thing::Variable(fastn_resolved::Variable {
    //                         name,
    //                         kind: fastn_resolved::KindData {
    //                             kind: val.kind(),
    //                             caption: false,
    //                             body: false,
    //                         },
    //                         mutable: false,
    //                         value: fastn_resolved::PropertyValue::Value {
    //                             value: val.to_owned(),
    //                             line_number: *line_number,
    //                             is_mutable: *is_mutable,
    //                         },
    //                         conditional_value: vec![],
    //                         line_number: *line_number,
    //                         is_static: !mutable,
    //                     }),
    //                     Some(fastn_resolved::PropertyValue::Reference { name, .. })
    //                     | Some(fastn_resolved::PropertyValue::Clone { name, .. }) => {
    //                         let (initial_thing, name) = doc.get_initial_thing(name, line_number)?;
    //                         if let Some(remaining) = name {
    //                             get_thing_(doc, line_number, remaining.as_str(), &initial_thing)?
    //                         } else {
    //                             initial_thing
    //                         }
    //                     }
    //                     _ => thing.clone(),
    //                 }
    //             }
    //             ftd::interpreter::Thing::OrType(fastn_resolved::OrType {
    //                 name, variants, ..
    //             }) => {
    //                 let variant = variants
    //                     .iter()
    //                     .find_or_first(|variant| variant.name().eq(&format!("{name}.{v}")))
    //                     .ok_or(ftd::interpreter::Error::ParseError {
    //                         message: format!("Cant't find `{v}` variant in `{name}` or-type"),
    //                         doc_id: doc.name.to_string(),
    //                         line_number,
    //                     })?;
    //                 variant.to_thing(doc.name, line_number)?
    //             }
    //             _ => {
    //                 return doc.err("not an or-type", thing, "get_thing", line_number);
    //             }
    //         };
    //         if let Some(remaining) = remaining {
    //             return get_thing_(doc, line_number, &remaining, &thing);
    //         }
    //         Ok(thing)
    //     }
    // }
}
