#![allow(dead_code)]

mod angle_text;
mod identifier;
mod qualified_identifier;
mod scanner;

use scanner::Scanner;

impl fastn_p1::ParseOutput {
    pub fn parse_v4(source: &str) -> fastn_p1::ParseOutput {
        let _scanner = scanner::Scanner::new(source);
        todo!()
    }
}
