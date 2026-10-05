use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

pub trait IntoBytes {

}

#[proc_macro_derive(IntoBytes)]
// pub fn derive_