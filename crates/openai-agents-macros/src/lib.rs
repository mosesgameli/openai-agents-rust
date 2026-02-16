//! Procedural macro for creating function tools

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, ReturnType};

/// Procedural macro for creating function tools
///
/// This macro transforms an async function into a Tool implementation.
///
/// # Example
///
/// ```rust,ignore
/// use openai_agents::function_tool;
///
/// #[function_tool]
/// async fn get_weather(city: String) -> String {
///     format!("The weather in {} is sunny", city)
/// }
/// ```
#[proc_macro_attribute]
pub fn function_tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;

    // Extract function parameters
    let params: Vec<_> = input
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(pat_type)
            } else {
                None
            }
        })
        .collect();

    // Extract return type
    let return_type = match &input.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    // Note: param_names and param_types will be used in future enhancements
    // for proper JSON schema generation and argument deserialization
    let _param_names: Vec<_> = params.iter().map(|p| &p.pat).collect();

    let _param_types: Vec<_> = params.iter().map(|p| &p.ty).collect();

    // Generate the tool struct name
    let tool_struct_name = syn::Ident::new(
        &format!("{}Tool", fn_name.to_string().to_uppercase()),
        fn_name.span(),
    );

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name(#(#params),*) -> #return_type {
            #fn_block
        }

        #[allow(non_camel_case_types)]
        #fn_vis struct #tool_struct_name;

        #[async_trait::async_trait]
        impl openai_agents::Tool for #tool_struct_name {
            fn name(&self) -> &str {
                stringify!(#fn_name)
            }

            fn description(&self) -> &str {
                // TODO: Extract from doc comments
                stringify!(#fn_name)
            }

            fn parameters_schema(&self) -> serde_json::Value {
                // TODO: Generate JSON schema from parameters
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                })
            }

            async fn execute(&self, args: serde_json::Value) -> openai_agents::Result<serde_json::Value> {
                // TODO: Deserialize args and call function properly
                // For now, just return a placeholder
                Ok(serde_json::json!("Tool executed"))
            }
        }
    };

    TokenStream::from(expanded)
}
