use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(IntoBytes, attributes(opcode))]
pub fn derive_into_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let opcode: syn::LitInt = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("opcode"))
        .unwrap()
        .parse_args()
        .unwrap();

    let tokens = quote! {
        impl crate::p2p::protocol::IntoBytes for #name {
            fn into_bytes(self) -> Vec<u8> {
                let mut buf: Vec<u8> = vec![#opcode];
                let bytes: Vec<u8> = rmp_serde::to_vec(&self).unwrap_or(vec![]);
                buf.extend_from_slice(&bytes[..]);
                buf
            }
            fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
                rmp_serde::from_slice(&bytes).map_err(|e| e.to_string())
            }
        }
    };

    tokens.into()
}
