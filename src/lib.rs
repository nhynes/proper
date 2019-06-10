//! Derives for converting primitives to and from simple enums and newtype structs.
//! This crate is useful for converting between Rust and C representations.
//!
//! ## Simple enums
//!
//! A _simple_ enum is one that does not have variants containing data. In other words, it's a
//! C-like/POD enum (as opposed to a tagged union).
//! Simple enums can be converted from unsigned integer types no larger than the `min_ty` and to
//! unsigned integer types no smaller than the `min_ty`.
//! The default `min_ty` is the smallest type that can fit the number of enum variants.
//! For example, an enum with 255 variants would have `min_ty = u8` but one with 256 would have
//! `min_ty = u16`. The `min_ty` can be configured using the `#[prim(ty = "<min_ty>")]` attribute.
//!
//! ### Example
//!
//! ```
//! #[macro_use]
//! extern crate proper;
//!
//! use std::convert::TryFrom;
//!
//! #[derive(Prim)]
//! enum EnumU8 {
//!     Variant0,
//!     Variant1,
//! }
//!
//! #[derive(Prim)]
//! #[prim(ty = "u16")]
//! enum EnumU16 {
//!     Variant0,
//!     Variant1,
//! }
//!
//! let e_u8 = EnumU8::try_from(42u8)?;
//! let e_u8 = EnumU8::try_from(42u16)?;   // won't compile!
//! let e_u16 = EnumU16::try_from(42u16)?; // compiles!
//!
//! let prim = e_u8 as u32;
//! let prim = e_u16 as u8;
//! ```
//!
//! ## Newtype structs
//!
//! Newtype structs can be created from an integer type with the same sign and bitwidth
//! no larger than the contained type.
//! Inversely, newtype structs can be converted to any integer type with the same sign and bitwidth
//! as the contained type.
//! For instance, `MyType(u16)` can be created from `u8` and `u16` but not `u32`; and converted to
//! `u16` and `u32`, but not `u8`.
//!
//! ### Example
//!
//! ```
//! #[macro_use]
//! extern crate proper;
//!
//! #[derive(Prim)]
//! struct FileDescriptor(u32);
//!
//! let fd = FileDescriptor::from(42u8);
//! let prim: u16 = fd.into(); // won't compile!
//! let prim: u32 = fd.into();
//! let prim: u64 = fd.into();
//! ```
#![feature(bind_by_move_pattern_guards, proc_macro_diagnostic, proc_macro_span)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use syn::spanned::Spanned;

#[macro_use]
mod util;
mod attr;
mod conversions;
mod int_ty;

use attr::PrimAttrs;
use int_ty::IntTy;

#[proc_macro_derive(Prim, attributes(prim))]
pub fn derive_prim(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let input_ident = &input.ident;
    let attrs = PrimAttrs::new(&input.attrs);

    let wrapper_ident = format_ident!("_IMPL_PRIM_FOR_{}", input_ident);

    let conversions = match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unnamed(ref fields),
            ..
        }) if fields.unnamed.len() == 1 => conversions::derive_newtype(input_ident, attrs, fields),
        syn::Data::Enum(syn::DataEnum { variants, .. })
            if variants.iter().all(|v| v.fields.iter().count() == 0) =>
        {
            conversions::derive_enum(input_ident, attrs, variants.iter().collect())
        }
        _ => {
            err!(input: "`Prim` can only be derived on newtype structs and simple enums.");
            quote!()
        }
    };

    (quote! {
        const #wrapper_ident: () = {
            use core::convert::TryFrom;

            #conversions
        };
    })
    .into()
}
