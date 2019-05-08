macro_rules! err {
    ($( $tok:ident ).+ : $fstr:literal$(,)? $( $arg:expr ),*) => {
        err!([error] $($tok).+ : $fstr, $($arg),*)
    };
    ([ $level:ident ] $( $tok:ident ).+ : $fstr:literal$(,)? $( $arg:expr ),*) => {
        $($tok).+.span().unwrap().$level(format!($fstr, $($arg),*)).emit();
    };
}

macro_rules! format_ident {
    ($fmt_str:literal, $($fmt_arg:expr),+) => {
        syn::Ident::new(&format!($fmt_str, $($fmt_arg),+), proc_macro2::Span::call_site())
    }
}
