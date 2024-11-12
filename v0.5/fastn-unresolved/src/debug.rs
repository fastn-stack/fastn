impl fastn_section::JDebug for fastn_unresolved::Import {
    fn debug(&self, _source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();

        let name = if self.package.0.is_empty() {
            self.module.0.to_string()
        } else {
            format!("{}/{}", self.package.0, self.module.0)
        };

        o.insert(
            "import".into(),
            match self.alias {
                Some(ref v) => format!("{name}=>{}", v.0),
                None => name,
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
