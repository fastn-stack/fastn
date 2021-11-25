fn main() {
    // let _args = fpm::Cli::from_args();
    let matches = clap::App::new("Fifthtry Package Manager")
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
        // Create output directory
        println!("Building...");
        std::fs::create_dir_all("./.build").expect("failed to create build folder");

        fpm::process_dir(
            std::env::current_dir()
                .expect("Panic1")
                .to_str()
                .expect("panic")
                .to_string(),
            0,
        );
    }
}
