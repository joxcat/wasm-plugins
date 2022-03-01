#![recursion_limit = "128"]

use proc_macro::TokenStream;
use syn::{parse::Parser, punctuated::Punctuated, token::Comma, ExprType, Ident};

/* For complex types */
mod complex_types;

#[proc_macro_attribute]
pub fn complex_export(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    complex_types::complex_export(attr, tokens)
}

/* For basic types */
mod standard_types;

#[proc_macro_attribute]
pub fn simple_export(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    standard_types::simple_export(attr, tokens)
}

/* To help with NativeFunction */
#[proc_macro]
pub fn impl_call_with_tuple(attrs: TokenStream) -> TokenStream {
    let attrs2 = proc_macro2::TokenStream::from(attrs);
    let parser = Punctuated::<Ident, Comma>::parse_terminated;

    let idents = parser.parse2(attrs2).expect("Could not parse macros attrs");

    let constraints = idents.iter().map(|ident| {
        syn::parse2::<ExprType>(quote::quote! { #ident: ::wasmer::FromToNativeWasmType }).unwrap()
    });
    let idents_idx = (0..idents.len()).map(syn::Index::from);

    match idents.len() {
        0 => quote::quote! {
            impl<Output> CallWithTuple<(), Output> for ::wasmer::NativeFunc<(), Output>
            where
                Output: ::wasmer::WasmTypeList,
            {
                fn call_with_tuple(&self, t: ()) -> Result<Output, ::wasmer::RuntimeError> {
                    self.call()
                }
            }
        },
        1 => quote::quote! {
            impl<Output, #idents> CallWithTuple<(#idents,), Output> for ::wasmer::NativeFunc<#idents, Output>
            where
                Output: ::wasmer::WasmTypeList,
                #(#constraints),*
            {
                fn call_with_tuple(&self, t: (#idents,)) -> Result<Output, ::wasmer::RuntimeError> {
                    self.call(#(t.#idents_idx),*)
                }
            }
        },
        _ => quote::quote! {
            impl<Output, #idents> CallWithTuple<(#idents), Output> for ::wasmer::NativeFunc<(#idents), Output>
            where
                Output: ::wasmer::WasmTypeList,
                #(#constraints),*
            {
                fn call_with_tuple(&self, t: (#idents)) -> Result<Output, ::wasmer::RuntimeError> {
                    self.call(#(t.#idents_idx),*)
                }
            }
        },
    }.into()
}
