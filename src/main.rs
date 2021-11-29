fn main() {
    let matches = clap::App::new("FTD Package Manager")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Shobhit S. <shobhit@fifthtry.com>")
        .about("Description...")
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            clap::SubCommand::with_name("build")
                .about("Builds the current directory")
                .version(env!("CARGO_PKG_VERSION"))
                .arg(
                    clap::Arg::with_name("base_dir")
                        .short("-d")
                        .long("base-dir")
                        .takes_value(true)
                        .help("Provide the base directory for building"),
                ),
        )
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("build") {
        fpm::build(matches.value_of("base_dir"));
    }
}
