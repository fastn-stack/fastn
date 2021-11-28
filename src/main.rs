fn main() {
    let matches = clap::App::new("FTD Package Manager")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Shobhit S. <shobhit@fifthtry.com>")
        .about("Description...")
        .arg(
            clap::Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            clap::SubCommand::with_name("build")
                .about("Builds the current directory")
                .version(env!("CARGO_PKG_VERSION")),
        )
        .get_matches();

    if matches.subcommand_matches("build").is_some() {
        fpm::build()
    }
}
