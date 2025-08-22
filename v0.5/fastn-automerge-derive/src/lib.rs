use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Type};

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
    
    let id52_field_name = &id52_field.ident;
    let id52_field_type = &id52_field.ty;
    
    // Generate document ID constructor function name
    let doc_id_fn_name = format_ident!("{}_id", struct_name.to_string().to_snake_case());
    
    let expanded = quote! {
        // Auto-include all necessary derives for document structs
        #[derive(Debug, Clone, PartialEq)]
        #[derive(fastn_automerge::Reconcile, fastn_automerge::Hydrate)]
        
        /// Document ID constructor for #struct_name
        pub fn #doc_id_fn_name(#id_field_name: &#id_field_type) -> fastn_automerge::DocumentId {
            let id_str = format!("/-/{}/{}", #id_field_name.id52(), stringify!(#struct_name).to_lowercase());
            fastn_automerge::DocumentId::from_string(&id_str)
                .expect("Generated document ID should be valid")
        }
        
        impl #struct_name {
            /// Load document from database
            pub fn load(
                db: &fastn_automerge::Db, 
                #id_field_name: &#id_field_type
            ) -> std::result::Result<Self, fastn_automerge::DocumentLoadError> {
                let doc_id = #doc_id_fn_name(#id_field_name);
                db.get(&doc_id).map_err(fastn_automerge::DocumentLoadError::Get)
            }
            
            /// Save document to database
            pub fn save(&self, db: &fastn_automerge::Db) -> std::result::Result<(), fastn_automerge::DocumentSaveError> {
                let doc_id = #doc_id_fn_name(&self.#id_field_name);
                if db.exists(&doc_id).map_err(fastn_automerge::DocumentSaveError::Exists)? {
                    db.update(&doc_id, self).map_err(fastn_automerge::DocumentSaveError::Update)
                } else {
                    db.create(&doc_id, self).map_err(fastn_automerge::DocumentSaveError::Create)
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}

fn find_document_id52_field(input: &DeriveInput) -> &Field {
    if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                // Look for #[document_id52] attribute
                for attr in &field.attrs {
                    if attr.path().is_ident("document_id52") {
                        return field;
                    }
                }
            }
        }
    }
    panic!("Document derive requires a field with #[document_id52] attribute");
}

// Helper trait for string conversion
trait ToSnakeCase {
    fn to_snake_case(&self) -> String;
}

impl ToSnakeCase for String {
    fn to_snake_case(&self) -> String {
        let mut result = String::new();
        for (i, c) in self.chars().enumerate() {
            if i > 0 && c.is_uppercase() {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        }
        result
    }
}