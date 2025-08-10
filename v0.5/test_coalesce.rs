use fastn_section::JDebug;

fn main() {
    let source = arcstr::ArcStr::from("color: black\ncolor if { dark-mode }: white\ncolor if { high-contrast }: yellow");
    let mut scanner = fastn_section::Scanner::new(fastn_section::Module::Main, &source);
    let headers = fastn_section::parser::headers(&mut scanner);
    println!("{}", serde_json::to_string_pretty(&headers.debug()).unwrap());
}