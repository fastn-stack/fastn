mod build;
mod render;
mod serve;

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
// fastn upload [--build] [--no-lint] <fifthtry-site-slug> (upload the current package)
// fastn clone <package-name>
pub enum UI {
    Terminal,
    Native,
    Browser,
}

#[derive(clap::Args)]
pub struct Render {
    /// Path to render (default: /)
    #[arg(default_value = "/")]
    pub path: String,
    /// Key-value pairs for rendering
    #[arg(skip = vec![])]
    pub key_values: Vec<(String, serde_json::Value)>,
    /// Action type
    #[arg(skip = fastn::Action::Read)]
    pub action: fastn::Action,
    /// Output type
    #[arg(skip)]
    pub output: Option<fastn::OutputRequested>,
    /// Browse mode
    #[arg(long)]
    pub browse: bool,
    /// Strict mode
    #[arg(long)]
    pub strict: bool,
    /// UI type
    #[arg(skip = UI::Terminal)]
    pub ui: UI,
    /// Offline mode
    #[arg(long)]
    pub offline: bool,
}

#[derive(clap::Args)]
pub struct Serve {
    /// Protocol to use
    #[arg(long, default_value = "http")]
    pub protocol: String,
    /// Address to listen on
    #[arg(long, default_value = "127.0.0.1:8000")]
    pub listen: std::net::SocketAddr,
    /// Watch for changes
    #[arg(long)]
    pub watch: bool,
    /// Build before serving
    #[arg(long)]
    pub build: bool,
    /// Offline mode
    #[arg(long)]
    pub offline: bool,
}

#[derive(clap::Args)]
pub struct Build {
    /// Offline mode
    #[arg(long)]
    pub offline: bool,
    /// Watch for changes
    #[arg(long)]
    pub watch: bool,
    /// Strict mode
    #[arg(long)]
    pub strict: bool,
}

#[derive(clap::Parser)]
#[command(name = "fastn")]
#[command(about = "A full-stack web development framework")]
#[command(version)]
pub enum Cli {
    /// Start the P2P networking node (default when no arguments)
    #[command(name = "run")]
    Run {
        /// Path to fastn home directory
        #[arg(long)]
        home: Option<std::path::PathBuf>,
    },
    /// Render pages to HTML
    Render(Render),
    /// Build the project
    Build(Build),
    /// Start development server
    Serve(Serve),
    /// Manage static files
    Static {
        /// Build static files
        #[arg(long)]
        build: bool,
        /// Offline mode
        #[arg(long)]
        offline: bool,
    },
    /// Run tests
    Test {
        /// Offline mode
        #[arg(long)]
        offline: bool,
    },
    /// Format FTD files
    Fmt {
        /// File to format (optional)
        file: Option<String>,
    },
    /// Upload to cloud
    Upload {
        /// Build before upload
        #[arg(long)]
        build: bool,
        /// Skip linting
        #[arg(long)]
        no_lint: bool,
        /// Upload slug
        slug: String,
    },
    /// Clone a repository
    Clone {
        /// Repository URL
        url: String,
    },
    /// Manage Automerge CRDT documents
    #[command(subcommand)]
    Automerge(fastn_automerge::cli::Commands),
}
