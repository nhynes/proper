use syn::spanned::Spanned;

use crate::{IntTy, PrimAttrs};

pub fn derive_newtype(
    input_ident: &syn::Ident,
    attrs: Option<PrimAttrs>,
    fields: &syn::FieldsUnnamed,
) -> proc_macro2::TokenStream {
    let newtype_field = fields.unnamed.first().expect("`len()` == 1");
    let newtype = match &newtype_field.value().ty {
        syn::Type::Path(syn::TypePath { path, .. }) => {
            let ty_ident = &path.segments.last().unwrap().value().ident;
            IntTy::new(ty_ident.clone())
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
                return quote! {
                    compile_error!("Your enum has way too many variants...")
                };
            },
            proc_macro2::Span::call_site(),
        ))
        .unwrap(),
    };
    let min_ty_ident = min_ty.ident();

    let mut last: Option<u64> = None;
    let from_converters = &variants
        .iter()
        .map(|f| {
            let ident = &f.ident;
            let value = match &f.discriminant {
                Some((_, syn::Expr::Lit(expr_lit))) => {
                    if let syn::Lit::Int(int) = &expr_lit.lit {
                        last = Some(int.value());
                        int.value()
                    } else {
                        return quote! {
                          compile_error!("Unexpected enum discriminator");
                        };
                    }
                }

                Some((_, _)) => {
                    return quote! {
                      compile_error!("Unexpected enum discriminator");
                    };
                }

                None => {
                    let next = last.map(|x| x + 1).unwrap_or(0);
                    last = Some(next);
                    next
                }
            };

            let i = syn::LitInt::new(value, syn::IntSuffix::None, proc_macro2::Span::call_site());
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
