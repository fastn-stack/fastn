mod build;
mod parse;
mod render;
mod serve;

pub use parse::parse;

// fastn <path> key=value
// or echo {json} | fastn <path>
// --method=GET | POST (stdin json means POST by default)
// --output=DATA | UI (default decided by the program)
//
// path can include full package name, in which case we will not look for package in current
// directory but will load package from the internet and store in ~/.fastn/<package-name>. the
// database for the app will be created in `~/.fastn/<package-name>/db.sqlite3`.
//
// other commands do not start with / etc., so do not look like path.
// `fastn` is the same as `fastn /`.
//
// if you want to browse the package instead, without creating local db, you can pass --browse.
//
// by default the UI will be rendered in terminal, but you can pass `--ui` to open native UI, and
// --browser to open in browser.
//
// `fastn www.foo.com` will run it offline, with local database etc. ~/.fastn/<domain> etc.
//
// # fastn build
//
// `fastn build` will download the packages from the internet and compile the JS files etc.,
// based on the changes in the current package. most of the commands below accept `--build` to
// do the compilation first and then do the command. else those commands are lazy and work off
// of the current compiled state of the package.
//
// fastn build --offline can be used to compile the package without downloading anything.
//
// fastn serve [--port=8080] [--watch] [--build] (serve the current package)
// fastn static [--build] (create static version of the site, issues warning if not all pages are static)
// fastn test (test the current package)
// fastn fmt
// fastn lint
// fastn upload [--build] [--no-lint] <fifthtry-site-slug> (upload the current package)
// fastn clone <package-name>
pub enum UI {
    Terminal,
    Native,
    Browser,
}

pub struct Render {
    pub path: String,
    // how to handle stdin?
    pub key_values: Vec<(String, serde_json::Value)>,
    pub action: fastn::Action,
    pub output: Option<fastn::OutputRequested>,
    pub browse: bool,
    pub ui: UI,
    pub offline: bool,
}

pub struct Serve {
    pub port: Option<u16>,
    pub watch: bool,
    pub build: bool,
    pub offline: bool,
}

pub struct Build {
    pub offline: bool,
    pub watch: bool,
    pub strict: bool,
}

pub enum Cli {
    Render(Render),
    Build(Build),
    Serve(Serve),
    Static {
        build: bool,
        offline: bool,
    },
    Test {
        offline: bool,
    },
    Fmt(Option<String>), // which file to format
    Upload {
        build: bool,
        no_lint: bool,
        slug: String,
    },
    Clone(String),
}
