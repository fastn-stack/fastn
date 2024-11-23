impl fastn_jdebug::JDebug for fastn_unresolved::AliasableIdentifier {
    fn debug(&self) -> serde_json::Value {
        match self.alias {
            Some(ref v) => format!("{}=>{}", self.name.str(), v.str()),
            None => self.name.str().to_string(),
        }
        .into()
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::ComponentInvocation {
    fn debug(&self) -> serde_json::Value {
        todo!()
    }
}

impl<U: fastn_jdebug::JDebug, R: fastn_jdebug::JDebug> fastn_jdebug::JDebug
    for fastn_unresolved::UR<U, R>
{
    fn debug(&self) -> serde_json::Value {
        todo!()
    }
}
