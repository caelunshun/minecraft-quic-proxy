//! Implements derives for the protocol Encode and Decode traits.

use darling::{FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields};

/// Options to encode a field.
#[derive(Default, Debug, FromField)]
#[darling(attributes(encoding), forward_attrs(allow, doc, cfg))]
#[darling(default)]
pub struct FieldOptions {
    /// Use varint-encoding for this field.
    varint: bool,
    /// Use varlong-encoding for this field.
    varlong: bool,
    /// Use the special angle encoding for this field.
    angle: bool,
    /// For an option field, prefix the field with a boolean
    /// to determine whether the field is present.
    bool_prefixed: bool,
    /// For a list field, how do we encode the length?
    length_prefix: Option<LengthPrefix>,
}

/// For a list field, how do we encode the length?
#[derive(Debug, FromMeta)]
pub enum LengthPrefix {
    /// Prefix with a varint.
    #[darling(rename = "varint")]
    VarInt,
    /// Infer the length from the remaining length of the stream.
    ///
    /// Only works for the last field of a packet.
    #[darling(rename = "inferred")]
    Inferred,
}

/// Options to encode an enum.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(encoding), forward_attrs(allow, doc, cfg))]
struct EnumOptions {
    /// How do we determine the discriminant?
    discriminant: Discriminant,
}

#[derive(Debug, FromMeta)]
enum Discriminant {
    /// Prefix the item with a varint.
    #[darling(rename = "varint")]
    VarInt,
    /// Prefix the item with a byte.
    #[darling(rename = "byte")]
    Byte,
    /// Prefix the item with a normal integer.
    #[darling(rename = "int")]
    Int,
}

/// Options to encode an enum variant.
#[derive(Debug, FromVariant)]
#[darling(attributes(encoding), forward_attrs(allow, doc, cfg))]
struct VariantOptions {
    id: i64,
}

#[derive(Debug)]
struct FieldInput {
    ident: Ident,
    get: TokenStream,
    options: FieldOptions,
}

#[derive(Debug)]
struct StructInput {
    fields: Vec<FieldInput>,
}

#[derive(Debug)]
struct EnumInput {
    variants: Vec<VariantInput>,
    options: EnumOptions,
}

#[derive(Debug)]
struct VariantInput {
    ident: Ident,
    fields: Vec<FieldInput>,
    bindings: Vec<Ident>,
    options: VariantOptions,
    fields_named: bool,
}

#[derive(Debug)]
enum Input {
    Struct(StructInput),
    Enum(EnumInput),
}

fn encode_field(field: &FieldInput) -> syn::Result<TokenStream> {
    let FieldInput { options, get, .. } = field;
    let num_set = options.bool_prefixed as u32
        + options.varint as u32
        + options.varlong as u32
        + options.length_prefix.is_some() as u32;
    if num_set > 1 {
        return Err(syn::Error::new(
            Span::call_site(),
            "at most one encoding option can be set",
        ));
    }

    let result = if options.varint {
        quote! {
            encoder.write_var_int(#get.try_into().unwrap_or(i32::MAX));
        }
    } else if options.varlong {
        quote! {
            encoder.write_var_long(#get.try_into().unwrap_or(i64::MAX));
        }
    } else if options.angle {
        quote! {
            encoder.write_angle(#get);
        }
    } else if options.bool_prefixed {
        quote! {
            encoder.write_bool(#get.is_some());
            if let Some(val) = &#get {
                crate::protocol::Encode::encode(val, encoder);
            }
        }
    } else if let Some(length_prefix) = &options.length_prefix {
        let encode_length = match length_prefix {
            LengthPrefix::Inferred => quote! {},
            LengthPrefix::VarInt => quote! {
                encoder.write_var_int(#get.len().try_into().unwrap_or(i32::MAX));
            },
        };

        quote! {
            #encode_length
            for item in &#get {
                crate::protocol::Encode::encode(item, encoder);
            }
        }
    } else {
        quote! {
            crate::protocol::Encode::encode(&#get, encoder);
        }
    };
    Ok(result)
}

fn encode_variant(variant: &VariantInput, parent: &EnumInput) -> syn::Result<TokenStream> {
    let write_discriminant = match &parent.options.discriminant {
        Discriminant::Byte => {
            let id = u8::try_from(variant.options.id).expect("ID overflow");
            quote! {
                encoder.write_u8(#id);
            }
        }
        Discriminant::Int => {
            let id = variant.options.id;
            quote! {
                encoder.write_u32(#id);
            }
        }
        Discriminant::VarInt => {
            let id = i32::try_from(variant.options.id).expect("ID overflow");
            quote! {
                encoder.write_var_int(#id);
            }
        }
    };

    let encode_fields = variant
        .fields
        .iter()
        .map(encode_field)
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        #write_discriminant
        #(#encode_fields)*
    })
}

fn derive_encode_struct(input: &StructInput) -> syn::Result<TokenStream> {
    let encode_fields = input
        .fields
        .iter()
        .map(encode_field)
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quote! {
        #(#encode_fields)*
    })
}

fn derive_encode_enum(input: &EnumInput) -> syn::Result<TokenStream> {
    let mut match_arms = Vec::new();

    for variant in &input.variants {
        let ident = &variant.ident;
        let bindings = &variant.bindings;
        let encode = encode_variant(variant, input)?;
        let binding = if bindings.is_empty() {
            quote! {}
        } else if !variant.fields_named {
            quote! { (__field) }
        } else {
            quote! {
                { #(#bindings),* }
            }
        };
        match_arms.push(quote! {
            Self::#ident #binding => {
                #encode
            }
        });
    }

    Ok(quote! {
        match self {
            #(#match_arms,)*
        }
    })
}

fn encode(input: &Input, ident: &Ident) -> syn::Result<TokenStream> {
    let encode = match input {
        Input::Struct(s) => derive_encode_struct(s)?,
        Input::Enum(e) => derive_encode_enum(e)?,
    };
    Ok(quote! {
        impl crate::protocol::Encode for #ident {
            fn encode(&self, encoder: &mut crate::protocol::Encoder) {
                #encode
            }
        }
    })
}

fn decode_field(field: &FieldInput) -> TokenStream {
    let FieldInput { options, ident, .. } = field;

    if options.varint {
        quote! {
            let #ident = decoder.read_var_int()?.try_into()?;
        }
    } else if options.varlong {
        quote! {
            let #ident = decoder.read_var_long()?.try_into()?;
        }
    } else if options.angle {
        quote! {
            let #ident = decoder.read_angle()?;
        }
    } else if options.bool_prefixed {
        quote! {
            let is_present = decoder.read_bool()?;
            let #ident = if is_present {
                Some(crate::protocol::Decode::decode(decoder)?)
            } else {
                None
            };
        }
    } else if let Some(length_prefix) = &options.length_prefix {
        match length_prefix {
            LengthPrefix::VarInt => quote! {let #ident = {
                let length = decoder.read_var_int()?;
                let mut #ident = Vec::new();
                for _ in 0..length {
                    #ident.push(crate::protocol::Decode::decode(decoder)?);
                }
                #ident
            };},
            LengthPrefix::Inferred => quote! {
                let mut #ident = Vec::new();
                while !decoder.is_finished() {
                    #ident.push(crate::protocol::Decode::decode(decoder)?);
                }
            },
        }
    } else {
        quote! {
            let #ident = crate::protocol::Decode::decode(decoder)?;
        }
    }
}

fn decode_struct(input: &StructInput) -> TokenStream {
    let decode_fields: Vec<_> = input.fields.iter().map(decode_field).collect();

    let init_fields: Vec<_> = input
        .fields
        .iter()
        .map(|FieldInput { ident, .. }| {
            quote! {
                #ident
            }
        })
        .collect();

    quote! {
        #(#decode_fields)*
        Ok(Self {
            #(#init_fields,)*
        })
    }
}

fn decode_variant(input: &VariantInput) -> TokenStream {
    let decode_fields: Vec<_> = input.fields.iter().map(decode_field).collect();

    let init_fields: Vec<_> = input
        .fields
        .iter()
        .map(|FieldInput { ident, .. }| {
            quote! {
                #ident
            }
        })
        .collect();

    let init = if init_fields.is_empty() {
        quote! {}
    } else if !input.fields_named {
        quote! { (#(#init_fields)*) }
    } else {
        quote! {
            {
                #(#init_fields,)*
            }
        }
    };

    let ident = &input.ident;
    quote! {
        #(#decode_fields)*
        Ok(Self::#ident #init)
    }
}

fn decode_enum(input: &EnumInput) -> TokenStream {
    let decode_discriminant = match &input.options.discriminant {
        Discriminant::VarInt => quote! { decoder.read_var_int()? },
        Discriminant::Byte => quote! { decoder.read_u8()? },
        Discriminant::Int => quote! { decoder.read_i32()? },
    };

    let mut match_arms = Vec::new();
    for variant in &input.variants {
        let decode = decode_variant(variant);
        let id = variant.options.id;
        match_arms.push(quote! {
            #id => {
                #decode
            }
        });
    }

    quote! {
        let discriminant = i64::from(#decode_discriminant);

        match discriminant {
            #(#match_arms,)*
            _ => Err(crate::protocol::DecodeError::Other(::anyhow::format_err!("invalid discriminant '{}'", discriminant))),
        }
    }
}

fn decode(input: &Input, derive_input: &DeriveInput) -> TokenStream {
    let ident = &derive_input.ident;
    let imp = match input {
        Input::Struct(s) => decode_struct(s),
        Input::Enum(e) => decode_enum(e),
    };

    quote! {
        impl crate::protocol::Decode for #ident {
            fn decode(decoder: &mut crate::protocol::Decoder) -> ::std::result::Result<Self, crate::protocol::DecodeError> {
                #imp
            }
        }
    }
}

fn get_input(input: &DeriveInput) -> syn::Result<Input> {
    match &input.data {
        Data::Struct(s) => get_struct_input(s).map(Input::Struct),
        Data::Enum(e) => get_enum_input(e, input).map(Input::Enum),
        Data::Union(u) => Err(syn::Error::new_spanned(
            u.union_token,
            "cannot derive Encode/Decode on a union",
        )),
    }
}

fn get_struct_input(s: &DataStruct) -> syn::Result<StructInput> {
    let mut fields = Vec::new();
    match &s.fields {
        Fields::Named(named) => {
            for field in &named.named {
                let options = FieldOptions::from_field(field)?;
                let ident = field.ident.as_ref().unwrap();
                fields.push(FieldInput {
                    get: quote! {
                        self.#ident
                    },
                    options,
                    ident: ident.clone(),
                });
            }
        }
        Fields::Unnamed(unnamed) => {
            return Err(syn::Error::new_spanned(
                &unnamed.unnamed,
                "structs with unnamed fields are unsupported",
            ))
        }
        Fields::Unit => {}
    }

    Ok(StructInput { fields })
}

fn get_enum_input(s: &DataEnum, input: &DeriveInput) -> syn::Result<EnumInput> {
    let options = EnumOptions::from_derive_input(input)?;
    let mut variants = Vec::new();

    for variant in &s.variants {
        let options = VariantOptions::from_variant(variant)?;

        let mut bindings = Vec::new();
        let mut fields = Vec::new();

        match &variant.fields {
            Fields::Named(named) => {
                for field in &named.named {
                    let ident = field.ident.as_ref().unwrap();
                    let options = FieldOptions::from_field(field)?;
                    fields.push(FieldInput {
                        get: quote! { (*#ident) },
                        options,
                        ident: ident.clone(),
                    });

                    bindings.push(ident.clone());
                }
            }
            Fields::Unnamed(unnamed) => {
                if unnamed.unnamed.len() > 1 {
                    return Err(syn::Error::new_spanned(
                        &unnamed.unnamed,
                        "more than one unnamed field in a variant is unsupported",
                    ));
                }
                let field = &unnamed.unnamed[0];
                let options = FieldOptions::from_field(field)?;
                fields.push(FieldInput {
                    get: quote! { *__field },
                    options,
                    ident: Ident::new("__field", Span::call_site()),
                });
                bindings.push(Ident::new("__field", Span::call_site()));
            }
            Fields::Unit => {}
        };

        variants.push(VariantInput {
            ident: variant.ident.clone(),
            fields,
            bindings,
            options,
            fields_named: matches!(variant.fields, Fields::Named(_)),
        });
    }

    Ok(EnumInput { variants, options })
}

pub fn derive_encode_on(derive_input: &DeriveInput) -> syn::Result<TokenStream> {
    let input = get_input(derive_input)?;
    encode(&input, &derive_input.ident)
}

pub fn derive_decode_on(derive_input: &DeriveInput) -> syn::Result<TokenStream> {
    let input = get_input(derive_input)?;
    Ok(decode(&input, derive_input))
}
