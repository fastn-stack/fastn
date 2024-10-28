/// example: `hello` | `hello ${world}` | `hello ${world} ${ -- foo: }` | `{ \n text text \n }`
pub fn ses(_scanner: &mut fastn_p1::parser::Scanner) -> Option<fastn_p1::SES> {
    todo!()
}

#[cfg(test)]
mod test {
    fastn_p1::tt!(super::ses);

    #[test]
    fn ses() {}
}
