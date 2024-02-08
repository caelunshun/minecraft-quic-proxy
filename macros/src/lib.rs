use syn::DeriveInput;

mod from_variants;
mod protocol;

#[proc_macro_derive(Encode, attributes(encoding))]
pub fn derive_encode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    protocol::derive_encode_on(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Decode, attributes(encoding))]
pub fn derive_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    protocol::derive_decode_on(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(FromVariants)]
pub fn derive_from_variants(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    from_variants::derive_from_variants_on(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
