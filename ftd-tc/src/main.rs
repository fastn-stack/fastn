fn main() {
    ftd_tc::parse_document_to_ast("-- import: foo", "foo").unwrap();
}
