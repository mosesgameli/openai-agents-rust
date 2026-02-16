//! Procedural macro for creating function tools

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

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

    // Extract description from doc comments
    let mut description = String::new();
    for attr in &input.attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(syn::MetaNameValue {
                value:
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }),
                ..
            }) = &attr.meta
            {
                let val = s.value().trim().to_string();
                if !description.is_empty() {
                    description.push(' ');
                }
                description.push_str(&val);
            }
        }
    }
    if description.is_empty() {
        description = fn_name.to_string();
    }

    // Extract function parameters for schema and execution
    let mut properties = Vec::new();
    let mut required = Vec::new();
    let mut param_names = Vec::new();
    let mut param_deserialization = Vec::new();

    for arg in &input.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let name = &pat_ident.ident;
                let name_str = name.to_string();

                // For now, assume everything is a string/deserializable
                properties.push(quote! {
                    #name_str: {
                        "type": "string"
                    }
                });
                required.push(quote! { #name_str });
                param_names.push(name);

                param_deserialization.push(quote! {
                    let #name = serde_json::from_value(args[#name_str].clone())
                        .map_err(|e| openai_agents::AgentError::tool_failed(stringify!(#fn_name), e.to_string()))?;
                });
            }
        }
    }

    // Generate the tool struct name (uppercase version of fn name + Tool)
    let tool_struct_name = syn::Ident::new(
        &format!("{}Tool", fn_name.to_string().to_uppercase()),
        fn_name.span(),
    );

    let expanded = quote! {
        #input

        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        #fn_vis struct #tool_struct_name;

        #[async_trait::async_trait]
        impl openai_agents::Tool for #tool_struct_name {
            fn name(&self) -> &str {
                stringify!(#fn_name)
            }

            fn description(&self) -> &str {
                #description
            }

            fn parameters_schema(&self) -> serde_json::Value {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        #(#properties),*
                    },
                    "required": [#(#required),*]
                })
            }

            async fn execute(&self, args: serde_json::Value) -> openai_agents::Result<serde_json::Value> {
                #(#param_deserialization)*
                let result = #fn_name(#(#param_names),*).await;
                Ok(serde_json::to_value(result).map_err(|e| openai_agents::AgentError::tool_failed(stringify!(#fn_name), e.to_string()))?)
            }
        }
    };

    TokenStream::from(expanded)
}
