use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;


#[proc_macro_derive(IntoBytes, attributes(opcode))]
pub fn derive_into_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let opcode: syn::LitInt = input.attrs.iter().find(|a| a.path().is_ident("opcode")).unwrap().parse_args().unwrap();

    let tokens = quote!{
        impl crate::p2p::protocol::IntoBytes for #name {
            fn into_bytes(self) -> Vec<u8> {
                let mut buf: Vec<u8> = vec![#opcode];
                buf.extend_from_slice(&rmp_serde::to_vec(&self).unwrap()[..]);
                buf
            }
            fn from_bytes(bytes: &[u8]) -> Self {
                let message: Self = rmp_serde::from_slice(&bytes).unwrap();
                message
            }
        }
    };

    return tokens.into()
}