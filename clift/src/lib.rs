#![deny(unused_crate_dependencies)]

extern crate self as clift;

pub mod commands;

pub mod api;
pub mod utils;

pub fn attach_cmd(cmd: clap::Command) -> clap::Command {
    cmd.subcommand(
        clap::Command::new("upload")
            .about("Uploads files in current directory to www.fifthtry.com.")
            .arg(clap::arg!(<"site-slug"> "The site-slug of this site.").required(true))
            .arg(clap::arg!(--file <FILE> "Only upload a single file.").required(false))
            .arg(clap::arg!(--folder <FOLDER> "Only upload a single folder.").required(false))
            .arg(clap::arg!(--"dry-run" "Do not actually upload anything.")),
    )
}

pub async fn upload(matches: &clap::ArgMatches) {
    if let Some(upload) = matches.subcommand_matches("upload") {
        let site = upload.get_one::<String>("site-slug").unwrap();
        let file = upload.get_one::<String>("file");
        let folder = upload.get_one::<String>("folder");
        let dry_run = *upload.get_one::<bool>("dry-run").unwrap_or(&false);

        if file.is_some() && folder.is_some() {
            eprintln!("both --file and --folder can not be specified");
            return;
        }

        if let Some(file) = file {
            if let Err(e) = clift::commands::upload_file(site, file, dry_run).await {
                eprintln!("Upload failed: {e}");
                std::process::exit(1);
            }
            return;
        }

        if let Some(folder) = folder {
            if let Err(e) = clift::commands::upload_folder(site, folder, dry_run).await {
                eprintln!("Upload failed: {e}");
                std::process::exit(1);
            }

            return;
        }

        if let Err(e) = clift::commands::upload_folder(site, "", dry_run).await {
            eprintln!("Upload failed: {e}");
            std::process::exit(1);
        }
    }
}
