use fastn_section::JDebug;

fn main() {
    // Simple test case - just two headers with same name
    let source = arcstr::ArcStr::from("color: black\ncolor: white");
    let mut scanner = fastn_section::Scanner::new(fastn_section::Module::Main, &source);
    
    if let Some(headers) = fastn_section::parser::headers(&mut scanner) {
        println!("Found {} headers", headers.len());
        for header in &headers {
            println!("Header '{}' has {} values", header.name.name.str(), header.values.len());
        }
        let json = headers.debug();
        println!("JSON: {}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        println!("No headers found");
    }
}