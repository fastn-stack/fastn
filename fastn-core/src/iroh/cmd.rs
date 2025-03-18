pub fn attach_cmd(cmd: clap::Command) -> clap::Command {
    cmd.subcommand(
        clap::Command::new("proxy")
            .about("Proxy to a fastn net fastn-http service")
            .arg(clap::arg!(<"id"> "The id of fastn-net http service.").required(true))
            .arg(clap::arg!(--port <PORT> "The port to listen on [default: first available port starting 8000]"))
    )
}
