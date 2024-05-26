// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, ItemFn};

// // Attribute-Like Macro
// #[proc_macro_attribute]
// pub fn log_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
//     // We ignore the attributes for this example, but they could be used if needed.
//     let func = parse_macro_input!(item as ItemFn);

//     let func_name = &func.sig.ident;
//     let block = &func.block;

//     let expanded = quote! {
//         fn #func_name() {
//             println!("Start running function: {}", stringify!(#func_name));
//             #block
//             println!("Success!");
//         }
//     };

//     TokenStream::from(expanded)
// }
