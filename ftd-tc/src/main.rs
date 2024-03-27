fn main() {
    ftd_tc::parse_document_to_ast("-- import: foo", &ftd_tc::DocumentID::new0("foo")).unwrap();
}
