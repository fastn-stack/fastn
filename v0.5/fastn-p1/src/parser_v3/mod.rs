mod module_doc;
mod scanner;
mod section;
mod utils;

use module_doc::module_doc;
use section::section;

impl fastn_p1::ParseOutput {
    pub fn new(_name: &str, source: &str) -> fastn_p1::ParseOutput {
        let mut scanner = scanner::Scanner::new(source.to_string());

        if module_doc(&mut scanner) {
            return scanner.output;
        }

        while section(&mut scanner) {}

        scanner.output
    }
}
