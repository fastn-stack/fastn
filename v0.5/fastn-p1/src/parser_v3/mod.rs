#![allow(dead_code)]

mod module_doc;
mod scanner;
mod section;
mod utils;

use module_doc::module_doc;
use section::section;

impl fastn_p1::ParseOutput {
    pub fn parse_v3(source: &str) -> fastn_p1::ParseOutput {
        let mut scanner = scanner::Scanner::new(source);
        let mut potential_errors: Vec<fastn_p1::Spanned<fastn_p1::SingleError>> = vec![];

        println!("this");
        module_doc(&mut scanner, &mut potential_errors);
        potential_errors.clear();
        println!("that:{}", scanner.is_done());

        let mut count = 0;
        while !scanner.is_done() && count < 100 {
            println!("here: {count}");
            section(&mut scanner, &mut potential_errors);
            potential_errors.clear();
            count += 1;
        }

        scanner.output
    }
}
