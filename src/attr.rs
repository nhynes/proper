use syn::spanned::Spanned;

use crate::IntTy;

pub struct PrimAttrs {
    pub min_ty: Option<IntTy>,
    pub err_ctor: Option<syn::Path>,
}

impl PrimAttrs {
    pub fn new(attrs: &[syn::Attribute]) -> Option<Self> {
        let mut has_errs = false;

        let mut min_ty = None;
        let mut err_ctor = None;

        for attr in attrs.iter() {
            if !attr.path.is_ident("prim") {
                continue;
            }
            let nested_metas = match attr.parse_meta() {
                Ok(syn::Meta::List(syn::MetaList { nested, .. })) => nested,
                Ok(_) => {
                    err!(attr: "Unrecognized meta item");
                    has_errs = true;
                    continue;
                }
                Err(e) => {
                    err!(attr: "Could not parse attribute meta: {}", e);
                    has_errs = true;
                    continue;
                }
            };

            for nested_meta in nested_metas {
                let meta = match nested_meta {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(meta)) => meta,
                    meta_item => {
                        err!(meta_item: "Unrecognized meta item: `{}`",
                             meta_item.span().unwrap().source_text().unwrap());
                        has_errs = true;
                        continue;
                    }
                };

                let lit = match &meta.lit {
                    syn::Lit::Str(lit_str) => lit_str,
                    lit => {
                        err!(lit: "The value of `{}` must be a string literal.", meta.ident);
                        has_errs = true;
                        continue;
                    }
                };

                if meta.ident == "ty" {
                    if min_ty.is_some() {
                        err!(meta: "Multiple `ty` attributes provided.");
                        has_errs = true;
                        continue;
                    }
                    min_ty = match lit.parse::<syn::Ident>().ok().and_then(IntTy::new) {
                        Some(ty) => Some(ty),
                        None => {
                            err!(lit: "The value of `{}` must be an integer type", meta.ident);
                            has_errs = true;
                            continue;
                        }
                    };
                } else if meta.ident == "error" {
                    if err_ctor.is_some() {
                        err!(meta: "Multiple `error` attributes provided.");
                        has_errs = true;
                        continue;
                    }
                    err_ctor = match lit.parse::<syn::Path>() {
                        Ok(ctor) => Some(ctor),
                        Err(_) => {
                            err!(lit: "The value of `{}` must be a path to a constructor",
                                 meta.ident);
                            has_errs = true;
                            continue;
                        }
                    };
                } else {
                    err!(meta.ident: "Unrecognized attribute `{}`", meta.ident);
                    has_errs = true;
                    continue;
                }
            }
        }

        if has_errs {
            None
        } else {
            Some(Self { min_ty, err_ctor })
        }
    }
}
