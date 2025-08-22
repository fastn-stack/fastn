pub fn run_command(cli: super::Cli) -> eyre::Result<()> {
    match cli.command {
        super::Commands::Init => {
            init_database(&cli.db)?;
            println!("Initialized database at {}", cli.db);
        }
        _ => {
            // For all other commands, open the existing database
            // WARNING: Using dummy entity ID for CLI - real apps should use actual entity ID52
            let db = fastn_automerge::Db::open(std::path::Path::new(&cli.db))?;

            match cli.command {
                super::Commands::Init => unreachable!(),
                super::Commands::Create { path, json, file } => {
                    let json_data = if let Some(file_path) = file {
                        super::utils::read_json_file(&file_path)?
                    } else if let Some(json_str) = json {
                        json_str
                    } else {
                        eprintln!("Error: Either provide JSON data or use --file option");
                        std::process::exit(1);
                    };
                    create_document(&db, &path, &json_data)?;
                    println!("Created document at {path}");
                }
                super::Commands::Get {
                    path,
                    pretty,
                    output,
                } => {
                    get_document(&db, &path, pretty, output.as_deref())?;
                }
                super::Commands::Update { path, json } => {
                    update_document(&db, &path, &json)?;
                    println!("Updated document at {path}");
                }
                super::Commands::Set { path, json } => {
                    set_document(&db, &path, &json)?;
                    println!("Set document at {path}");
                }
                super::Commands::Delete { path, confirm } => {
                    delete_document(&db, &path, confirm)?;
                    println!("Deleted document at {path}");
                }
                super::Commands::List { prefix, details } => {
                    list_documents(&db, prefix.as_deref(), details)?;
                }
                super::Commands::Clean { force } => {
                    clean_database(&db, force)?;
                }
                super::Commands::History {
                    path,
                    commit_hash,
                    short,
                } => {
                    show_history(&db, &path, commit_hash.as_deref(), short)?;
                }
                super::Commands::Info { path } => {
                    show_info(&db, &path)?;
                }
            }
        }
    }

    Ok(())
}

fn init_database(db_path: &str) -> eyre::Result<()> {
    // WARNING: Using dummy entity for CLI - real apps should use actual PublicKey
    let dummy_entity_str = super::utils::get_dummy_cli_entity_id();
    let dummy_entity = std::str::FromStr::from_str(&dummy_entity_str)?;
    let path = std::path::Path::new(db_path);
    let _db = fastn_automerge::Db::init(path, &dummy_entity)?;
    Ok(())
}

fn create_document(db: &fastn_automerge::Db, path: &str, json: &str) -> eyre::Result<()> {
    // Validate JSON first
    let _value = super::utils::parse_json(json)?;

    // Create typed path with validation
    let doc_id = fastn_automerge::DocumentPath::from_string(path)?;

    // For CLI simplicity, store JSON as string with metadata
    let mut data = std::collections::HashMap::new();
    data.insert("json_data".to_string(), json.to_string());
    data.insert("content_type".to_string(), "application/json".to_string());

    db.create(&doc_id, &data)?;
    Ok(())
}

fn get_document(
    db: &fastn_automerge::Db,
    path: &str,
    pretty: bool,
    output: Option<&str>,
) -> eyre::Result<()> {
    let doc_id = fastn_automerge::DocumentPath::from_string(path)?;
    let data: std::collections::HashMap<String, String> = db.get(&doc_id)?;

    // Extract JSON data
    let json_str = data
        .get("json_data")
        .ok_or_else(|| eyre::eyre!("Document does not contain JSON data"))?;

    let json_output = if pretty {
        // Parse and re-format for pretty printing
        let value = super::utils::parse_json(json_str)?;
        serde_json::to_string_pretty(&value).unwrap()
    } else {
        json_str.clone()
    };

    if let Some(output_path) = output {
        std::fs::write(output_path, &json_output)?;
        println!("Output written to {output_path}");
    } else {
        println!("{json_output}");
    }

    Ok(())
}

fn update_document(db: &fastn_automerge::Db, path: &str, json: &str) -> eyre::Result<()> {
    // Validate JSON first
    let _value = super::utils::parse_json(json)?;

    // Create typed path
    let doc_id = fastn_automerge::DocumentPath::from_string(path)?;

    // Update with new JSON data
    let mut data = std::collections::HashMap::new();
    data.insert("json_data".to_string(), json.to_string());
    data.insert("content_type".to_string(), "application/json".to_string());

    db.update(&doc_id, &data)?;
    Ok(())
}

fn set_document(db: &fastn_automerge::Db, path: &str, json: &str) -> eyre::Result<()> {
    // Validate JSON first
    let _value = super::utils::parse_json(json)?;

    // Create typed path
    let doc_id = fastn_automerge::DocumentPath::from_string(path)?;

    // Prepare data
    let mut data = std::collections::HashMap::new();
    data.insert("json_data".to_string(), json.to_string());
    data.insert("content_type".to_string(), "application/json".to_string());

    // Set = create if not exists, update if exists
    if db.exists(&doc_id)? {
        db.update(&doc_id, &data)?;
    } else {
        db.create(&doc_id, &data)?;
    }
    Ok(())
}

fn delete_document(db: &fastn_automerge::Db, path: &str, confirm: bool) -> eyre::Result<()> {
    let doc_id = fastn_automerge::DocumentPath::from_string(path)?;

    if !confirm && !super::utils::confirm_action(&format!("Delete document at {path}?")) {
        println!("Cancelled");
        return Ok(());
    }

    db.delete(&doc_id)?;
    Ok(())
}

fn list_documents(
    db: &fastn_automerge::Db,
    prefix: Option<&str>,
    details: bool,
) -> eyre::Result<()> {
    let documents = db.list(prefix)?;

    if details {
        for path in documents {
            let doc_id = fastn_automerge::DocumentPath::from_string(&path)?;
            if db.exists(&doc_id)? {
                println!("{path}");
            }
        }
    } else {
        for path in documents {
            println!("{path}");
        }
    }

    Ok(())
}

fn clean_database(db: &fastn_automerge::Db, force: bool) -> eyre::Result<()> {
    if !force && !super::utils::confirm_action("This will delete ALL documents. Are you sure?") {
        println!("Cancelled");
        return Ok(());
    }

    let count = db.clear()?;
    println!("Deleted {count} documents");
    Ok(())
}

fn show_history(
    db: &fastn_automerge::Db,
    path: &str,
    commit_hash: Option<&str>,
    short: bool,
) -> eyre::Result<()> {
    let doc_id = fastn_automerge::DocumentPath::from_string(path)?;
    let history = db.history(&doc_id, commit_hash)?;

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

fn show_info(db: &fastn_automerge::Db, path: &str) -> eyre::Result<()> {
    let doc_id = fastn_automerge::DocumentPath::from_string(path)?;

    if !db.exists(&doc_id)? {
        return Err(eyre::eyre!("Document not found: {path}"));
    }

    let history = db.history(&doc_id, None)?;

    println!("Document: {path}");
    println!("Created by: {}", history.created_alias);
    println!("Updated at: {}", history.updated_at);
    println!("Heads: {}", history.heads.join(", "));
    println!("Total edits: {}", history.edits.len());

    Ok(())
}
