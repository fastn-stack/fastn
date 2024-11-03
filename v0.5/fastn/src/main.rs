fn main() {
    // fastn <path> key=value
    // or echo {json} | fastn <path>
    // --method=GET | POST (stdin json means POST by default)
    // --output=DATA | UI (default decided by the program)
    //
    // other commands do not start with / etc., so do not look like path.
    // `fastn` is the same as `fastn /`.
    //
    // `fastn browse www.foo.com` etc can be used to view any site.
    // `fastn browse` is our terminal / UI based browser.
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
    println!("Hello, world!");
}
