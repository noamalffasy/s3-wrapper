use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

extern crate proc_macro;

mod error;

#[proc_macro_derive(S3Error, attributes(error))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let error_args =
        match error::ErrorOpts::from_derive_input(&parse_macro_input!(input as DeriveInput)) {
            Ok(error_args) => error_args,
            Err(err) => return TokenStream::from(err.write_errors()),
        };

    TokenStream::from(quote!(#error_args))
}
