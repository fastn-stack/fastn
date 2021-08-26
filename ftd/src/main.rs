pub fn main() {
    // use std::io::Write;
    // let id = match std::env::args().nth(1) {
    //     Some(v) => v,
    //     None => {
    //         eprintln!("Usage: ftd <filename.ftd>");
    //         return;
    //     }
    // };
    //
    // let mut lib = ftd::p2::TestLibrary::default();
    // lib.libs.insert(
    //     "fifthtry/ft".to_string(),
    //     std::fs::read_to_string("../ft.ftd").expect("failed to read ft.ftd"),
    // );
    //
    // let b = match ftd::p2::Document::from(
    //     id.as_str(),
    //     std::fs::read_to_string(id.as_str())
    //         .expect("cant read file")
    //         .as_str(),
    //     &lib,
    // ) {
    //     Ok(v) => v,
    //     Err(e) => {
    //         eprintln!("failed to parse {}: {:?}", id, &e);
    //         return;
    //     }
    // };
    //
    // let mut f =
    //     std::fs::File::create(id.replace(".ftd", ".html")).expect("failed to create .html file");
    //
    // // TODO: indent things properly
    // f.write_all(
    //     std::fs::read_to_string("ftd.html")
    //         .expect("cant read ftd.html")
    //         .replace(
    //             "___ftd___",
    //             b.main.to_node().to_html(&Default::default()).as_str(),
    //         )
    //         .as_bytes(),
    // )
    // .expect("failed to write to .html file");
}
