use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields};

pub fn derive_from_variants_on(input: &DeriveInput) -> syn::Result<TokenStream> {
    let Data::Enum(en) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "only enums are supported",
        ));
    };

    let enum_ident = &input.ident;

    let mut impls = Vec::new();
    for variant in &en.variants {
        let variant_ident = &variant.ident;
        if let Fields::Unnamed(fields) = &variant.fields {
            if fields.unnamed.len() == 1 {
                let Field { ty, .. } = &fields.unnamed[0];
                impls.push(quote! {
                    impl From<#ty> for #enum_ident {
                        fn from(value: #ty) -> Self {
                            Self::#variant_ident(value)
                        }
                    }
                });
            }
        }
    }

    Ok(quote! {
        #(#impls)*
    })
}
