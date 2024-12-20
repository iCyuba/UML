use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

const ANIMATED_PROPERTY: &str = stringify!(AnimatedProperty);

#[proc_macro_derive(AnimatedElement)]
pub fn animated_element_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = if let Data::Struct(data_struct) = input.data {
        let updates: Vec<_> = match data_struct.fields {
            Fields::Named(fields_named) => fields_named
                .named
                .iter()
                .filter_map(|field| {
                    if let Type::Path(type_path) = &field.ty {
                        if type_path
                            .path
                            .segments
                            .iter()
                            .any(|segment| segment.ident == ANIMATED_PROPERTY)
                        {
                            let ident = &field.ident;
                            return Some(quote! { self.#ident.animate() });
                        }
                    }
                    None
                })
                .collect(),
            _ => Vec::new(),
        };

        quote! {
            impl #name {
                pub fn animate(&mut self) -> bool {
                    #(#updates)|*
                }
            }
        }
    } else {
        quote! {}
    };

    expanded.into()
}
