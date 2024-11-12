impl fastn_section::JDebug for fastn_unresolved::Import {
    fn debug(&self, _source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();

        o.insert(
            "import".into(),
            match self.alias {
                Some(ref v) => format!("{}{}=>{}", self.package.0, self.module.0, v.0),
                None => format!("{}{}", self.package.0, self.module.0),
            }
            .into(),
        );

        // if let Some(ref v) = self.exports {
        //     o.insert("exports".into(), v.debug(source));
        // }
        // if let Some(ref v) = self.exposing {
        //     o.insert("exposing".into(), v.debug(source));
        // }
        serde_json::Value::Object(o)
    }
}
