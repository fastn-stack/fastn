mod module_doc;
mod scanner;
mod utils;

use module_doc::module_doc;

impl fastn_p1::ParseOutput {
    pub fn new(_name: &str, source: &str) -> fastn_p1::ParseOutput {
        let mut scanner = scanner::Scanner::new(source.to_string());

        if module_doc(&mut scanner) {
            return scanner.output;
        }

        scanner.output
    }
}
