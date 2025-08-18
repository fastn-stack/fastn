pub fn run_command(cli: super::Cli) -> fastn_automerge::Result<()> {
    match cli.command {
        super::Commands::Init => {
            init_database(&cli.db)?;
            println!("Initialized database at {}", cli.db);
        }
        super::Commands::Create { path, json, file } => {
            let json_data = if let Some(file_path) = file {
                super::utils::read_json_file(&file_path)?
            } else if let Some(json_str) = json {
                json_str
            } else {
                eprintln!("Error: Either provide JSON data or use --file option");
                std::process::exit(1);
            };
            create_document(&cli.db, &path, &json_data)?;
            println!("Created document at {path}");
        }
        super::Commands::Get {
            path,
            pretty,
            output,
        } => {
            get_document(&cli.db, &path, pretty, output.as_deref())?;
        }
        super::Commands::Update { path, json } => {
            update_document(&cli.db, &path, &json)?;
            println!("Updated document at {path}");
        }
        super::Commands::Set { path, json } => {
            set_document(&cli.db, &path, &json)?;
            println!("Set document at {path}");
        }
        super::Commands::Delete { path, confirm } => {
            delete_document(&cli.db, &path, confirm)?;
            println!("Deleted document at {path}");
        }
        super::Commands::List { prefix, details } => {
            list_documents(&cli.db, prefix.as_deref(), details)?;
        }
        super::Commands::Clean { force } => {
            clean_database(&cli.db, force)?;
        }
        super::Commands::History {
            path,
            commit_hash,
            short,
        } => {
            show_history(&cli.db, &path, commit_hash.as_deref(), short)?;
        }
        super::Commands::Info { path } => {
            show_info(&cli.db, &path)?;
        }
    }

    Ok(())
}

fn init_database(db_path: &str) -> fastn_automerge::Result<()> {
    let actor_id = super::utils::get_actor_id();
    let path = std::path::Path::new(db_path);
    let _db = fastn_automerge::Db::init_with_actor(path, actor_id)?;
    Ok(())
}

fn create_document(db_path: &str, path: &str, json: &str) -> fastn_automerge::Result<()> {
    let db = super::utils::open_db(db_path)?;

    // Validate JSON first
    let _value = super::utils::parse_json(json)?;

    // For CLI simplicity, store JSON as string with metadata
    let mut data = std::collections::HashMap::new();
    data.insert("json_data".to_string(), json.to_string());
    data.insert("content_type".to_string(), "application/json".to_string());

    db.create(path, &data)
}

fn get_document(
    db_path: &str,
    path: &str,
    pretty: bool,
    output: Option<&str>,
) -> fastn_automerge::Result<()> {
    let db = super::utils::open_db(db_path)?;
    let data: std::collections::HashMap<String, String> = db.get(path)?;

    // Extract JSON data
    let json_str = data.get("json_data").ok_or_else(|| {
        super::utils::json_error("Document does not contain JSON data".to_string())
    })?;

    let json_output = if pretty {
        // Parse and re-format for pretty printing
        let value = super::utils::parse_json(json_str)?;
        serde_json::to_string_pretty(&value).unwrap()
    } else {
        json_str.clone()
    };

    if let Some(output_path) = output {
        std::fs::write(output_path, &json_output).map_err(|e| {
            super::utils::json_error(format!("Failed to write to file {output_path}: {e}"))
        })?;
        println!("Output written to {output_path}");
    } else {
        println!("{json_output}");
    }

    Ok(())
}

fn update_document(db_path: &str, path: &str, json: &str) -> fastn_automerge::Result<()> {
    let db = super::utils::open_db(db_path)?;

    // Validate JSON first
    let _value = super::utils::parse_json(json)?;

    // Update with new JSON data
    let mut data = std::collections::HashMap::new();
    data.insert("json_data".to_string(), json.to_string());
    data.insert("content_type".to_string(), "application/json".to_string());

    db.update(path, &data)
}

fn set_document(db_path: &str, path: &str, json: &str) -> fastn_automerge::Result<()> {
    let db = super::utils::open_db(db_path)?;

    // Validate JSON first
    let _value = super::utils::parse_json(json)?;

    // Prepare data
    let mut data = std::collections::HashMap::new();
    data.insert("json_data".to_string(), json.to_string());
    data.insert("content_type".to_string(), "application/json".to_string());

    // Set = create if not exists, update if exists
    if db.exists(path)? {
        db.update(path, &data)
    } else {
        db.create(path, &data)
    }
}

fn delete_document(db_path: &str, path: &str, confirm: bool) -> fastn_automerge::Result<()> {
    let db = super::utils::open_db(db_path)?;

    if !confirm && !super::utils::confirm_action(&format!("Delete document at {path}?")) {
        println!("Cancelled");
        return Ok(());
    }

    db.delete(path)
}

fn list_documents(
    db_path: &str,
    prefix: Option<&str>,
    details: bool,
) -> fastn_automerge::Result<()> {
    let db = super::utils::open_db(db_path)?;
    let documents = db.list(prefix)?;

    if details {
        for path in documents {
            if db.exists(&path)? {
                println!("{path}");
                // Could add more details like creation time, size, etc.
            }
        }
    } else {
        for path in documents {
            println!("{path}");
        }
    }

    Ok(())
}

fn clean_database(db_path: &str, force: bool) -> fastn_automerge::Result<()> {
    let db = super::utils::open_db(db_path)?;

    if !force && !super::utils::confirm_action("This will delete ALL documents. Are you sure?") {
        println!("Cancelled");
        return Ok(());
    }

    let count = db.clear()?;
    println!("Deleted {count} documents");
    Ok(())
}

fn show_history(
    db_path: &str,
    path: &str,
    commit_hash: Option<&str>,
    short: bool,
) -> fastn_automerge::Result<()> {
    let db = super::utils::open_db(db_path)?;
    let history = db.history(path, commit_hash)?;

    println!("History for {}", history.path);
    println!("Created by: {}", history.created_alias);
    println!("Updated at: {}", history.updated_at);
    println!("Heads: {}", history.heads.join(", "));
    println!();

    if short {
        println!("{} edits total", history.edits.len());
    } else {
        for edit in history.edits {
            println!("Edit #{}: {}", edit.index, edit.hash);
            println!("  Actor: {}", edit.actor_id);
            println!("  Timestamp: {}", edit.timestamp);
            if let Some(msg) = edit.message {
                println!("  Message: {msg}");
            }
            println!("  Operations: {} ops", edit.operations.len());
            for op in edit.operations {
                println!("    {op:?}");
            }
            println!();
        }
    }

    Ok(())
}

fn show_info(db_path: &str, path: &str) -> fastn_automerge::Result<()> {
    let db = super::utils::open_db(db_path)?;

    if !db.exists(path)? {
        return Err(Box::new(fastn_automerge::Error::NotFound(path.to_string())));
    }

    let history = db.history(path, None)?;

    println!("Document: {path}");
    println!("Created by: {}", history.created_alias);
    println!("Updated at: {}", history.updated_at);
    println!("Heads: {}", history.heads.join(", "));
    println!("Total edits: {}", history.edits.len());

    Ok(())
}
