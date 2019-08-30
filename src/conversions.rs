use syn::spanned::Spanned;

use crate::{IntTy, PrimAttrs};

pub fn derive_newtype(
    input_ident: &syn::Ident,
    attrs: Option<PrimAttrs>,
    fields: &syn::FieldsUnnamed,
) -> proc_macro2::TokenStream {
    let newtype_field = fields.unnamed.first().expect("`len()` == 1");
    let newtype = match &newtype_field.ty {
        syn::Type::Path(syn::TypePath { path, .. }) if path.get_ident().is_some() => {
            IntTy::new(path.get_ident().unwrap().clone())
        }
        _ => None,
    };
    if newtype.is_none() {
        err!(newtype_field: "Newtype struct must contain an integer type");
        return quote!();
    }
    let newtype = newtype.unwrap();
    let newtype_ident = newtype.ident();

    if let Some(attrs) = attrs {
        if let Some(min_ty) = &attrs.min_ty {
            let emit_span = min_ty
                .span()
                .join(newtype.span())
                .expect("newtype span should exist");
            if *min_ty == newtype {
                emit_span.warning("``ty` attribute is unneded on newtype structs.")
            } else {
                emit_span.error(format!(
                    "Mismatched type: expected `{}` but found `{}`",
                    min_ty.span().source_text().unwrap(),
                    newtype.span().source_text().unwrap()
                ))
            }
            .emit();
        }
    }

    let froms = newtype
        .subtypes()
        .iter()
        .map(|ty| {
            quote! {
                impl From<#ty> for #input_ident {
                    fn from(prim: #ty) -> Self {
                        Self(prim as #newtype_ident)
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let tos = newtype
        .supertypes()
        .iter()
        .map(|ty| {
            quote! {
                impl From<#input_ident> for #ty {
                    fn from(newtype: #input_ident) -> #ty {
                        newtype.0 as #ty
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #(#froms)*

        #(#tos)*
    }
}

pub fn derive_enum(
    input_ident: &syn::Ident,
    attrs: Option<PrimAttrs>,
    variants: Vec<&syn::Variant>,
) -> proc_macro2::TokenStream {
    let min_ty = match attrs {
        Some(PrimAttrs {
            min_ty: Some(min_ty),
            ..
        }) => min_ty,
        _ => IntTy::new(syn::Ident::new(
            if variants.len() <= <u8>::max_value() as usize {
                "u8"
            } else if variants.len() <= <u16>::max_value() as usize {
                "u16"
            } else if variants.len() <= <u32>::max_value() as usize {
                "u32"
            } else if variants.len() <= <u64>::max_value() as usize {
                "u64"
            } else {
                err!(input_ident: "Your enum has way too many variants...");
                return quote!();
            },
            proc_macro2::Span::call_site(),
        ))
        .unwrap(),
    };
    let min_ty_ident = min_ty.ident();

    let mut next = 0;
    let from_converters = &variants
        .iter()
        .map(|f| {
            let ident = &f.ident;
            let cur = match &f.discriminant {
                Some((_, syn::Expr::Lit(expr_lit))) => {
                    if let syn::Lit::Int(int) = &expr_lit.lit {
                        match int.base10_parse() {
                            Ok(int) => int,
                            Err(_) => {
                                err!(int: "{} is not a number", int);
                                return quote!();
                            }
                        }
                    } else {
                        err!(expr_lit: "Enum variant must be an integer.");
                        return quote!();
                    }
                }

                Some((_, expr)) => {
                    err!(expr: "Enum variant must be an integer.");
                    return quote!();
                }

                None => next,
            };

            let i = syn::LitInt::new(&format!("{}", cur), proc_macro2::Span::call_site());
            next = cur + 1;
            quote! {
                #i => { #input_ident::#ident }
            }
        })
        .collect::<Vec<_>>();

    let froms = min_ty
        .subtypes()
        .iter()
        .map(|ty| {
            quote! {
                impl TryFrom<#ty> for #input_ident {
                    type Error = ();
                    fn try_from(prim: #ty) -> Result<Self, Self::Error> {
                        Ok(match prim as #min_ty_ident {
                            #(#from_converters)*
                            _ => return Err(())
                        })
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    return quote! {
        #(#froms)*
    };
}
