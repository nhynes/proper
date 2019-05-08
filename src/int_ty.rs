#[derive(Debug)]
pub struct IntTy {
    ident: syn::Ident,
    signed: bool,
    bits: u8,
}

const BITS: &[u8] = &[8, 16, 32, 64];

impl PartialEq for IntTy {
    fn eq(&self, other: &Self) -> bool {
        self.signed == other.signed && self.bits == other.bits
    }
}

impl IntTy {
    pub fn new(ident: syn::Ident) -> Option<Self> {
        let ident_str = ident.to_string();
        let signed = match &ident_str[0..1] {
            "i" => true,
            "u" => false,
            _ => return None,
        };
        let bits = match ident_str[1..].parse::<u8>() {
            Ok(bits) if bits == 8 || bits == 16 || bits == 32 || bits == 64 => bits,
            _ => return None,
        };
        Some(Self {
            ident,
            signed,
            bits,
        })
    }

    pub fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    pub fn span(&self) -> proc_macro::Span {
        self.ident.span().unwrap()
    }

    pub fn subtypes(&self) -> Vec<syn::Ident> {
        self.get_int_tys(<u8>::le)
    }

    pub fn supertypes(&self) -> Vec<syn::Ident> {
        self.get_int_tys(<u8>::ge)
    }

    fn get_int_tys(&self, comparator: fn(&u8, &u8) -> bool) -> Vec<syn::Ident> {
        let prefix = if self.signed { "i" } else { "u" };
        BITS.iter()
            .filter_map(|bits| {
                if comparator(bits, &self.bits) {
                    Some(syn::parse_str::<syn::Ident>(&format!("{}{}", prefix, bits)).unwrap())
                } else {
                    None
                }
            })
            .collect()
    }
}
