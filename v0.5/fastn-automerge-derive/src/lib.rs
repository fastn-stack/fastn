use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Field, Fields, parse_macro_input};

/// Derive macro for fastn-automerge document structs
///
/// Generates:
/// - `load(db, id) -> Result<Self, DocumentLoadError>`
/// - `save(&self, db) -> Result<(), DocumentSaveError>`
/// - Document ID constructor function
///
/// Usage:
/// ```rust
/// #[derive(Document)]
/// #[document_path("/-/aliases/{id52}/readme")]
/// struct AliasReadme {
///     #[document_id52]
///     alias: PublicKey,
///     name: String,
/// }
/// ```
#[proc_macro_derive(Document, attributes(document_id52, document_path))]
pub fn derive_document(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let id52_field = find_document_id52_field(&input);
    let document_path = find_document_path_attribute(&input);

    // Handle both cases: with and without document_id52 field
    let (id52_field_name, id52_field_type, has_id52_field) = if let Some(field) = id52_field {
        (Some(&field.ident), Some(&field.ty), true)
    } else {
        (None, None, false)
    };

    // Generate document path method name
    let doc_path_fn_name = format_ident!("document_path");

    let (path_fn_signature, path_generation) = if has_id52_field {
        // With ID52 field - function takes parameter
        let field_name = id52_field_name.unwrap();
        let field_type = id52_field_type.unwrap();

        let path_gen = if let Some(template) = &document_path {
            quote! { let path_str = #template.replace("{id52}", &#field_name.id52()); }
        } else {
            quote! { let path_str = format!("/-/{}/{}", stringify!(#struct_name).to_lowercase(), #field_name.id52()); }
        };

        (
            quote! { pub fn #doc_path_fn_name(#field_name: &#field_type) -> fastn_automerge::DocumentPath },
            path_gen,
        )
    } else {
        // Without ID52 field - static path
        let path_str = document_path
            .as_ref()
            .expect("Document path attribute required for structs without document_id52 field");
        let path_gen = quote! { let path_str = #path_str; };

        (
            quote! { pub fn #doc_path_fn_name() -> fastn_automerge::DocumentPath },
            path_gen,
        )
    };

    let load_signature = if has_id52_field {
        let field_name = id52_field_name.unwrap();
        let field_type = id52_field_type.unwrap();
        quote! { pub fn load(db: &fastn_automerge::Db, #field_name: &#field_type) -> std::result::Result<Self, fastn_automerge::db::GetError> }
    } else {
        quote! { pub fn load(db: &fastn_automerge::Db) -> std::result::Result<Self, fastn_automerge::db::GetError> }
    };

    let load_call = if has_id52_field {
        let field_name = id52_field_name.unwrap();
        quote! { let doc_path = Self::#doc_path_fn_name(#field_name); }
    } else {
        quote! { let doc_path = Self::#doc_path_fn_name(); }
    };

    let save_call = if has_id52_field {
        let field_name = id52_field_name.unwrap();
        quote! { let doc_path = Self::#doc_path_fn_name(&self.#field_name); }
    } else {
        quote! { let doc_path = Self::#doc_path_fn_name(); }
    };

    let expanded = quote! {
        impl #struct_name {
            /// Get the document path for this struct type
            #path_fn_signature {
                #path_generation
                fastn_automerge::DocumentPath::from_string(&path_str)
                    .expect("Generated document path should be valid")
            }
            /// Load document from database
            #load_signature {
                #load_call
                db.get(&doc_path).map_err(|e| e)
            }

            /// Create new document in database (fails if exists)
            pub fn create(&self, db: &fastn_automerge::Db) -> std::result::Result<(), fastn_automerge::db::CreateError> {
                #save_call
                db.create(&doc_path, self)
            }

            /// Update existing document in database (fails if not exists)
            pub fn update(&self, db: &fastn_automerge::Db) -> std::result::Result<(), fastn_automerge::db::UpdateError> {
                #save_call
                db.update(&doc_path, self)
            }

            /// Save document to database (create if not exists, update if exists)
            pub fn save(&self, db: &fastn_automerge::Db) -> std::result::Result<(), fastn_automerge::db::SaveError> {
                #save_call
                if db.exists(&doc_path).map_err(fastn_automerge::db::SaveError::Exists)? {
                    db.update(&doc_path, self).map_err(fastn_automerge::db::SaveError::Update)
                } else {
                    db.create(&doc_path, self).map_err(fastn_automerge::db::SaveError::Create)
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
