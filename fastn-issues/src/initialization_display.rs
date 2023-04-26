use fastn_issues::initialization::*;

pub fn display_initialisation_error(e: &InitializePackageError) {
    match e {
        InitializePackageError::FastnFTDError { source } => display_fastn_ftd_error(source),
        InitializePackageError::InitializeDBError { source } => display_initialize_db_error(source),
    }
}

fn display_fastn_ftd_error(e: &FastnFTDError) {
    match e {
        FastnFTDError::ReadFTDFile { source } => match source {
            FileAsStringError::FileDoesNotExist { .. } => {}
            _ => todo!(),
        },
        FastnFTDError::ParseFASTNFile { .. } => {
            todo!()
        }
        FastnFTDError::StorePackageName { .. } => {
            todo!()
        }
    }
}

fn display_initialize_db_error(e: &InitializeDBError) {
    match e {
        InitializeDBError::OpenDBConnection { .. } => {
            todo!()
        }
        InitializeDBError::CreateTables { .. } => {
            todo!()
        }
    }
}
