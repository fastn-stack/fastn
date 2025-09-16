use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Main function attribute macro for fastn applications with context support
#[proc_macro_attribute]
pub fn main(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    let user_fn_name = syn::Ident::new("__fastn_user_main", proc_macro2::Span::call_site());
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    
    quote! {
        #(#fn_attrs)*
        #fn_vis fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
            // Initialize tokio runtime
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?
                .block_on(async {
                    // Global context automatically created
                    
                    // Call user's main function
                    let result = #user_fn_name().await;
                    
                    result
                })
        }
        
        async fn #user_fn_name() -> std::result::Result<(), Box<dyn std::error::Error>> #fn_block
    }.into()
}