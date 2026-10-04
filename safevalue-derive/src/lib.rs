//! Derive macros for [safevalue](https://docs.rs/safevalue).
//!
//! Don't depend on this crate directly: the macros are re-exported by
//! `safevalue`, and only work together with it.

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Implements `safevalue::MarkerData` for a type, so it can be put into a
/// `safevalue::SafeHolder`. The holder hands out the value itself.
///
/// See the documentation of `safevalue::MarkerData`.
#[proc_macro_derive(MarkerData)]
pub fn derive_marker_data(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) =
        input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::safevalue::MarkerData
            for #name #type_generics #where_clause
        {
            type Data = Self;

            #[inline(always)]
            fn from_data(data: Self) -> Self { data }

            #[inline(always)]
            fn data(&self) -> &Self { self }

            #[inline(always)]
            fn data_mut(&mut self) -> &mut Self { self }

            #[inline(always)]
            fn into_data(self) -> Self { self }
        }
    }
    .into()
}
