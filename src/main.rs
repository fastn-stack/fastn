use std::env::current_dir;
use std::fs::metadata;

pub use ftd;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "fpm", about = "Fifthtry package manager usage")]
struct Cli {
    /// The pattern to look for Set speed
    #[structopt(short = "c", long, default_value = "FPM.ftd")]
    config_file: String,

    /// The path to the file to read
    #[structopt(short, long)]
    debug: bool,
}

fn main() {
    let _args = Cli::from_args();

    // Create output directory
    std::fs::create_dir_all("./build").expect("failed to create build folder");

    let total_processed = process_dir(
        current_dir()
            .expect("Panic1")
            .to_str()
            .unwrap_or_else(|| todo!("Panic2"))
            .to_string(),
    );
}

fn process_dir(directory: String) -> u32 {
    let mut count: u32 = 0;
    for entry in std::fs::read_dir(&directory).expect("?//") {
        let e = entry.expect("Panic: Doc not found");
        let md = metadata(e.path()).unwrap();
        if md.is_dir() {
            // Iterate the children
            count += process_dir(
                e.path()
                    .to_str()
                    .unwrap_or_else(|| todo!("Panic"))
                    .to_string(),
            );
        } else if e
            .path()
            .to_str()
            .unwrap_or_else(|| "no matches")
            .ends_with(".ftd")
        {
            // process the document
            let doc = std::fs::read_to_string(e.path().to_str().unwrap_or_else(|| todo!()))
                .expect("cant read file");
            write(
                // e.file_name().to_str().unwrap_or_else(|| todo!()).replace(".ftd", "").as_str(),
                e.file_name().to_str().unwrap_or_else(|| todo!()),
                doc,
            );
            count += 1;
        }
    }
    count
}

fn write(id: &str, doc: String) {
    use std::io::Write;

    let lib = ftd::ExampleLibrary {};
    let b = match ftd::p2::Document::from(id, &*doc, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", id, &e);
            return;
        }
    };

    let mut f = std::fs::File::create(format!("./build/{}", id.replace(".ftd", ".html")))
        .expect("failed to create .html file");

    let doc = b.to_rt("main", id);

    let ftd_js = std::fs::read_to_string("ftd.js").expect("ftd.js not found");

    f.write_all(
        std::fs::read_to_string("ftd.html")
            .expect("cant read ftd.html")
            .replace(
                "__ftd_data__",
                serde_json::to_string_pretty(&doc.data)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace(
                "__ftd_external_children__",
                serde_json::to_string_pretty(&doc.external_children)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace("__ftd__", b.html("main", id).as_str())
            .replace("__ftd_js__", ftd_js.as_str())
            .as_bytes(),
    )
    .expect("failed to write to .html file");
}
