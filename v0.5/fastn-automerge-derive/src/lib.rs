use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, parse_macro_input};

/// Derive macro for fastn-automerge document structs
///
/// Generates three different APIs based on your document structure:
///
/// ## 1. Template with `{id52}` placeholder
/// 
/// Most convenient for entity-specific documents:
/// 
/// ```ignore
/// #[derive(Document, Reconcile, Hydrate)]
/// #[document_path("/-/users/{id52}/profile")]
/// struct UserProfile {
///     #[document_id52] 
///     user_id: fastn_id52::PublicKey,
///     name: String,
/// }
/// 
/// // Generated API:
/// // UserProfile::load(db, &user_id) -> Result<UserProfile, GetError>
/// // profile.save(db) -> Result<(), SaveError>
/// // UserProfile::document_list(db) -> Result<Vec<DocumentPath>, ListError> ← NEW!
/// // UserProfile::document_path(&user_id) -> DocumentPath
/// ```
///
/// ## 2. Template without `{id52}` (singleton)
/// 
/// For global/singleton documents:
/// 
/// ```ignore
/// #[derive(Document, Reconcile, Hydrate)]
/// #[document_path("/-/app/config")]
/// struct AppConfig {
///     version: String,
///     features: Vec<String>,
/// }
/// 
/// // Generated API:
/// // AppConfig::load(db) -> Result<AppConfig, GetError>
/// // config.save(db) -> Result<(), SaveError>
/// // AppConfig::document_path() -> DocumentPath
/// // No document_list() - only one instance
/// ```
///
/// ## 3. No template (maximum flexibility)
/// 
/// Requires explicit paths for every operation:
/// 
/// ```ignore
/// #[derive(Document, Reconcile, Hydrate)]
/// struct FlexibleDoc {
///     #[document_id52]
///     id: fastn_id52::PublicKey,
///     data: String,
/// }
/// 
/// // Generated API:
/// // FlexibleDoc::load(db, &path) -> Result<FlexibleDoc, GetError>
/// // doc.save(db, &path) -> Result<(), SaveError>
/// // No document_path() or document_list() - paths are explicit
/// ```
///
/// ## Pattern Matching Details
///
/// `document_list()` uses exact DNSSEC32 validation:
/// - Template: `"/-/users/{id52}/notes"` 
/// - Matches: `"/-/users/abc123...xyz/notes"` (exactly 52 alphanumeric chars)
/// - Rejects: `"/-/users/short/notes"`, `"/-/users/has-dashes/notes"`
#[proc_macro_derive(Document, attributes(document_id52, document_path))]
pub fn derive_document(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let id52_field = find_document_id52_field(&input);
    let document_path = find_document_path_attribute(&input);

    if let Some(template) = document_path {
        // Template-based API: has document_path attribute
        generate_template_based_api(struct_name, id52_field, &template)
    } else {
        // Path-based API: no document_path attribute  
        generate_path_based_api(struct_name, id52_field)
    }
}

fn generate_template_based_api(
    struct_name: &syn::Ident,
    id52_field: Option<&Field>,
    template: &str,
) -> TokenStream {
    let has_id52_field = id52_field.is_some();
    
    if has_id52_field {
        let field = id52_field.unwrap();
        let field_name = &field.ident;
        let field_type = &field.ty;

        let expanded = quote! {
            impl #struct_name {
                /// Get the document path for this instance
                pub fn document_path(#field_name: &#field_type) -> fastn_automerge::DocumentPath {
                    let path_str = #template.replace("{id52}", &#field_name.id52());
                    fastn_automerge::DocumentPath::from_string(&path_str)
                        .expect("Generated document path should be valid")
                }

                /// Load document from database
                pub fn load(db: &fastn_automerge::Db, #field_name: &#field_type) -> std::result::Result<Self, fastn_automerge::db::GetError> {
                    let doc_path = Self::document_path(#field_name);
                    db.get_impl(&doc_path)
                }

                /// Create new document in database (fails if exists)
                pub fn create(&self, db: &fastn_automerge::Db) -> std::result::Result<(), fastn_automerge::db::CreateError> {
                    let doc_path = Self::document_path(&self.#field_name);
                    db.create_impl(&doc_path, self)
                }

                /// Update existing document in database (fails if not exists)
                pub fn update(&self, db: &fastn_automerge::Db) -> std::result::Result<(), fastn_automerge::db::UpdateError> {
                    let doc_path = Self::document_path(&self.#field_name);
                    db.update_impl(&doc_path, self)
                }

                /// Save document to database (create if not exists, update if exists)
                pub fn save(&self, db: &fastn_automerge::Db) -> std::result::Result<(), fastn_automerge::db::SaveError> {
                    let doc_path = Self::document_path(&self.#field_name);
                    if db.exists(&doc_path).map_err(fastn_automerge::db::SaveError::Exists)? {
                        db.update_impl(&doc_path, self).map_err(fastn_automerge::db::SaveError::Update)
                    } else {
                        db.create_impl(&doc_path, self).map_err(fastn_automerge::db::SaveError::Create)
                    }
                }
            }
        };
        
        // Only add document_list if template contains {id52}
        let list_fn = if template.contains("{id52}") {
            quote! {
                impl #struct_name {
                    /// List all document paths for this type
                    pub fn document_list(db: &fastn_automerge::Db) -> std::result::Result<Vec<fastn_automerge::DocumentPath>, fastn_automerge::db::ListError> {
                        // Extract prefix for SQL LIKE query
                        let prefix = if let Some(pos) = #template.find("{id52}") {
                            &#template[..pos]
                        } else {
                            #template
                        };
                        
                        // Get all paths with this prefix
                        let all_paths = db.list(Some(prefix))?;
                        
                        // Filter to match exact pattern with 52-char ID52
                        let matching_paths: std::result::Result<Vec<_>, _> = all_paths
                            .into_iter()
                            .filter(|path| {
                                // Validate that this path matches our exact template structure
                                Self::path_matches_template(path, #template)
                            })
                            .map(|p| fastn_automerge::DocumentPath::from_string(&p))
                            .collect();
                            
                        matching_paths.map_err(|_| fastn_automerge::db::ListError::Database(
                            rusqlite::Error::InvalidPath("Invalid document path returned from database".into())
                        ))
                    }

                    /// Check if a path matches our template with valid ID52
                    fn path_matches_template(path: &str, template: &str) -> bool {
                        // Split template at {id52} placeholder
                        if let Some(id52_pos) = template.find("{id52}") {
                            let prefix = &template[..id52_pos];
                            let suffix = &template[id52_pos + 6..]; // Skip "{id52}"
                            
                            // Check prefix and suffix match
                            if !path.starts_with(prefix) || !path.ends_with(suffix) {
                                return false;
                            }
                            
                            // Extract the ID part and validate it's exactly 52 alphanumeric chars
                            let id_start = prefix.len();
                            let id_end = path.len() - suffix.len();
                            
                            if id_end <= id_start {
                                return false;
                            }
                            
                            let id_part = &path[id_start..id_end];
                            id_part.len() == 52 && id_part.chars().all(|c| c.is_alphanumeric())
                        } else {
                            // No {id52} in template, exact match
                            path == template
                        }
                    }
                }
            }
        } else {
            quote! {} // No list function for singleton documents
        };
        
        let combined = quote! {
            #expanded
            #list_fn
        };
        
        TokenStream::from(combined)
    } else {
        // Singleton document with template but no ID field
        let expanded = quote! {
            impl #struct_name {
                /// Get the document path for this type
                pub fn document_path() -> fastn_automerge::DocumentPath {
                    fastn_automerge::DocumentPath::from_string(#template)
                        .expect("Template document path should be valid")
                }

                /// Load document from database
                pub fn load(db: &fastn_automerge::Db) -> std::result::Result<Self, fastn_automerge::db::GetError> {
                    let doc_path = Self::document_path();
                    db.get_impl(&doc_path)
                }

                /// Create new document in database (fails if exists)
                pub fn create(&self, db: &fastn_automerge::Db) -> std::result::Result<(), fastn_automerge::db::CreateError> {
                    let doc_path = Self::document_path();
                    db.create_impl(&doc_path, self)
                }

                /// Update existing document in database (fails if not exists)
                pub fn update(&self, db: &fastn_automerge::Db) -> std::result::Result<(), fastn_automerge::db::UpdateError> {
                    let doc_path = Self::document_path();
                    db.update_impl(&doc_path, self)
                }

                /// Save document to database (create if not exists, update if exists)
                pub fn save(&self, db: &fastn_automerge::Db) -> std::result::Result<(), fastn_automerge::db::SaveError> {
                    let doc_path = Self::document_path();
                    if db.exists(&doc_path).map_err(fastn_automerge::db::SaveError::Exists)? {
                        db.update_impl(&doc_path, self).map_err(fastn_automerge::db::SaveError::Update)
                    } else {
                        db.create_impl(&doc_path, self).map_err(fastn_automerge::db::SaveError::Create)
                    }
                }
            }
        };
        TokenStream::from(expanded)
    }
}

fn generate_path_based_api(
    struct_name: &syn::Ident,
    _id52_field: Option<&Field>,
) -> TokenStream {
    // Path-based API: all functions require explicit DocumentPath parameter
    
    let expanded = quote! {
        impl #struct_name {
            /// Load document from database using explicit path
            pub fn load(db: &fastn_automerge::Db, path: &fastn_automerge::DocumentPath) -> std::result::Result<Self, fastn_automerge::db::GetError> {
                db.get_impl(path)
            }

            /// Create new document in database (fails if exists)
            pub fn create(&self, db: &fastn_automerge::Db, path: &fastn_automerge::DocumentPath) -> std::result::Result<(), fastn_automerge::db::CreateError> {
                db.create_impl(path, self)
            }

            /// Update existing document in database (fails if not exists)
            pub fn update(&self, db: &fastn_automerge::Db, path: &fastn_automerge::DocumentPath) -> std::result::Result<(), fastn_automerge::db::UpdateError> {
                db.update_impl(path, self)
            }

            /// Save document to database (create if not exists, update if exists)
            pub fn save(&self, db: &fastn_automerge::Db, path: &fastn_automerge::DocumentPath) -> std::result::Result<(), fastn_automerge::db::SaveError> {
                if db.exists(path).map_err(fastn_automerge::db::SaveError::Exists)? {
                    db.update_impl(path, self).map_err(fastn_automerge::db::SaveError::Update)
                } else {
                    db.create_impl(path, self).map_err(fastn_automerge::db::SaveError::Create)
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn find_document_id52_field(input: &DeriveInput) -> Option<&Field> {
    if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                // Look for #[document_id52] attribute
                for attr in &field.attrs {
                    if attr.path().is_ident("document_id52") {
                        return Some(field);
                    }
                }
            }
        }
    }
    None
}

fn find_document_path_attribute(input: &DeriveInput) -> Option<String> {
    for attr in &input.attrs {
        if attr.path().is_ident("document_path") {
            // Parse the attribute value: #[document_path("/-/aliases/{id52}/readme")]
            if let Ok(lit) = attr.parse_args::<syn::LitStr>() {
                return Some(lit.value());
            }
        }
    }
    None
}