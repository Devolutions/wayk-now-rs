#![no_std]

extern crate alloc;
extern crate proc_macro;
extern crate proc_macro2;

use alloc::vec::Vec;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Add;
use syn::{Attribute, Data, Fields, Generics, Ident, Lifetime, LifetimeDef, Lit, LitInt, Meta, Type};

mod parsed {
    use alloc::vec::Vec;

    pub enum Type<'a> {
        Struct(Struct<'a>),
        EnumWithFallback(EnumWithFallback<'a>),
        MetaEnum(MetaEnum<'a>),
    }

    // == Basic struct == //

    pub struct Struct<'a> {
        pub name: &'a syn::Ident,
        pub generics: &'a syn::Generics,
        pub fields: Vec<Field<'a>>,
    }

    pub struct Field<'a> {
        pub decode_ignore: bool,
        pub encode_ignore: bool,
        pub name: &'a syn::Ident,
        pub ty: &'a syn::Type,
    }

    // == Trivial Enum with fallback == //

    pub struct EnumWithFallback<'a> {
        pub name: &'a syn::Ident,
        pub underlying_repr: &'a syn::Type,
        pub variants: Vec<VariantWithValue<'a>>,
        pub fallback_variant: &'a syn::Ident,
    }

    pub struct VariantWithValue<'a> {
        pub ident: &'a syn::Ident,
        pub value: syn::LitInt,
    }

    // == Meta Enum == //

    pub struct MetaEnum<'a> {
        pub name: &'a syn::Ident,
        pub generics: &'a syn::Generics,
        pub meta: syn::Meta,
        pub variants: Vec<MetaEnumVariant<'a>>,
        pub fallback_variant_ident: &'a syn::Ident,
    }

    pub struct MetaEnumVariant<'a> {
        pub decode_ignore: bool,
        pub encode_ignore: bool,
        pub name: &'a syn::Ident,
        pub field_type: &'a syn::Type,
    }
}

#[proc_macro_derive(Encode, attributes(meta_enum, encode_ignore, value, fallback))]
pub fn encode_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("failed to parse input");
    impl_trait(&ast, impl_encode)
}

fn impl_encode(ty: parsed::Type<'_>) -> TokenStream {
    match ty {
        parsed::Type::Struct(data) => {
            let ty = data.name;
            let (impl_generics, ty_generics, where_clause) = data.generics.split_for_impl();

            let fields: Vec<&Ident> = data
                .fields
                .iter()
                .filter(|field| !field.encode_ignore)
                .map(|field| field.name)
                .collect();

            let types: Vec<&Type> = data
                .fields
                .iter()
                .filter(|field| !field.encode_ignore)
                .map(|field| field.ty)
                .collect();

            let expanded = quote! {
                impl #impl_generics ::wayk_proto::serialization::Encode for #ty #ty_generics #where_clause {
                    fn expected_size() -> ::wayk_proto::serialization::ExpectedSize {
                        use ::wayk_proto::serialization::ExpectedSize;
                        ExpectedSize::Known( #(
                            if let ExpectedSize::Known(v) = <#types as ::wayk_proto::serialization::Encode>::expected_size() {
                                v
                            } else {
                                return ExpectedSize::Variable;
                            }
                        )+* )
                    }

                    fn encoded_len(&self) -> usize {
                        #(
                            self.#fields.encoded_len()
                        )+*
                    }

                    fn encode_into<W: ::wayk_proto::io::NoStdWrite>(&self, writer: &mut W) -> ::core::result::Result<(), ::wayk_proto::error::ProtoError> {
                        use ::wayk_proto::error::{ProtoErrorKind, ProtoErrorResultExt as _};
                        #(
                            self.#fields.encode_into(writer)
                                .chain(ProtoErrorKind::Encoding(stringify!(#ty)))
                                .or_else_desc(|| format!("couldn't encode {}::{}", stringify!(#ty), stringify!(#fields)))?;
                        )*
                        Ok(())
                    }
                }
            };

            expanded.into()
        }
        parsed::Type::MetaEnum(data) => {
            let ty = data.name;
            let (impl_generics, ty_generics, where_clause) = data.generics.split_for_impl();
            let fallback_variant_ident = data.fallback_variant_ident;

            let variants: Vec<&Ident> = data
                .variants
                .iter()
                .filter(|variant| !variant.encode_ignore)
                .map(|variant| variant.name)
                .collect();

            let expanded = quote! {
                impl #impl_generics ::wayk_proto::serialization::Encode for #ty #ty_generics #where_clause {
                    fn expected_size() -> ::wayk_proto::serialization::ExpectedSize {
                        ::wayk_proto::serialization::ExpectedSize::Variable
                    }

                    fn encoded_len(&self) -> usize {
                        match self {
                            #(
                                Self::#variants(msg) => msg.encoded_len(),
                            )*
                            Self::#fallback_variant_ident(msg) => msg.len(),
                        }
                    }

                    fn encode_into<W: ::wayk_proto::io::NoStdWrite>(&self, writer: &mut W) -> ::core::result::Result<(), ::wayk_proto::error::ProtoError> {
                        use ::wayk_proto::error::{ProtoError, ProtoErrorKind, ProtoErrorResultExt as _};
                        match self {
                            #(
                                Self::#variants(msg) => msg
                                    .encode_into(writer)
                                    .chain(ProtoErrorKind::Encoding(stringify!(#ty)))
                                    .or_desc(concat!("couldn't encode ", stringify!(#variants)," message")),
                            )*
                            Self::#fallback_variant_ident(msg) => writer.write_all(msg)
                                .map_err(ProtoError::from)
                                .chain(ProtoErrorKind::Encoding(stringify!(#ty)))
                                .or_desc("couldn't encode custom message"),
                        }
                    }
                }
            };

            expanded.into()
        }
        parsed::Type::EnumWithFallback(data) => {
            let ty = data.name;
            let underlying_repr = data.underlying_repr;
            let variants = data.variants;
            let fallback_variant = data.fallback_variant;

            let idents: Vec<&Ident> = variants.iter().map(|variant| variant.ident).collect();
            let values: Vec<&LitInt> = variants.iter().map(|variant| &variant.value).collect();

            let expanded = quote! {
                impl ::wayk_proto::serialization::Encode for #ty {
                    fn expected_size() -> ::wayk_proto::serialization::ExpectedSize {
                        ::wayk_proto::serialization::ExpectedSize::Known(::core::mem::size_of::<#underlying_repr>())
                    }

                    fn encoded_len(&self) -> usize {
                        ::core::mem::size_of::<#underlying_repr>()
                    }

                    fn encode_into<W: ::wayk_proto::io::NoStdWrite>(
                        &self,
                        writer: &mut W,
                    ) -> ::core::result::Result<(), ::wayk_proto::error::ProtoError> {
                        <#underlying_repr>::encode_into(&(#underlying_repr::from(*self)), writer)
                    }
                }

                impl ::core::convert::From<#ty> for #underlying_repr {
                    fn from(
                        v: #ty,
                    ) -> #underlying_repr {
                        match v {
                            #(
                                #ty::#idents => #values,
                            )*
                            #ty::#fallback_variant(inner) => inner,
                        }
                    }
                }

            };

            expanded.into()
        }
    }
}

#[proc_macro_derive(Decode, attributes(meta_enum, decode_ignore, value, fallback))]
pub fn decode_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("failed to parse input");
    impl_trait(&ast, impl_decode)
}

fn build_decode_impl_generics(generics: &Generics) -> TokenStream2 {
    let decode_lt = {
        let lt = Lifetime::new("'dec", Span::call_site());

        let mut bounds = Punctuated::<Lifetime, Add>::new();
        for bounded_lt in generics.lifetimes() {
            bounds.push(bounded_lt.lifetime.clone());
        }

        let mut lt_def = LifetimeDef::new(lt);
        lt_def.bounds = bounds;

        lt_def
    };

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();

    quote! {
        <#decode_lt, #(#lifetimes),* #(#type_params)+*>
    }
}

fn impl_decode(enc_dec_ty: parsed::Type<'_>) -> TokenStream {
    match enc_dec_ty {
        parsed::Type::Struct(data) => {
            let ty = data.name;

            let impl_generics = build_decode_impl_generics(data.generics);
            let (_, ty_generics, where_clause) = data.generics.split_for_impl();

            let fields_ty = data
                .fields
                .iter()
                .filter(|field| !field.decode_ignore)
                .map(|field| field.ty)
                .collect::<Vec<&Type>>();
            let fields = data
                .fields
                .iter()
                .filter(|field| !field.decode_ignore)
                .map(|field| field.name)
                .collect::<Vec<&Ident>>();
            let ignored_fields = data
                .fields
                .iter()
                .filter(|field| field.decode_ignore)
                .map(|field| field.name)
                .collect::<Vec<&Ident>>();

            let expanded = quote! {
                impl #impl_generics ::wayk_proto::serialization::Decode<'dec> for #ty #ty_generics #where_clause {
                    fn decode_from(cursor: &mut ::wayk_proto::io::Cursor<'dec>) -> ::core::result::Result<Self, ::wayk_proto::error::ProtoError> {
                        use ::wayk_proto::error::{ProtoErrorResultExt as _, ProtoErrorKind};
                        Ok(Self {
                            #(
                                #fields: <#fields_ty as ::wayk_proto::serialization::Decode>::decode_from(cursor)
                                    .chain(ProtoErrorKind::Decoding(stringify!(#ty)))
                                    .or_desc(concat!(
                                        "couldn't decode ",
                                        stringify!(#fields_ty),
                                        " into ",
                                        stringify!(#ty), "::", stringify!(#fields)
                                    ))?,
                            )*
                            #(
                                #ignored_fields: ::core::default::Default::default(),
                            )*
                        })
                    }
                }
            };

            expanded.into()
        }
        parsed::Type::MetaEnum(data) => {
            let ty = data.name;
            let generics = data.generics;
            let fallback_variant_ident = data.fallback_variant_ident;

            let subtype_enum_ty = if let Meta::NameValue(name) = data.meta {
                if let Lit::Str(s) = name.lit {
                    Ident::new(&s.value(), Span::call_site())
                } else {
                    panic!("wrong literal in `meta_enum` attribute parameter. Expected a string literal for the subtype enum.");
                }
            } else {
                panic!(r#"wrong meta for `meta_enum`. Expected a name value (eg: meta_enum = "...")."#);
            };

            let variants: Vec<&Ident> = data
                .variants
                .iter()
                .filter(|variant| !variant.decode_ignore)
                .map(|variant| variant.name)
                .collect();
            let variants_field_ty: Vec<&Type> = data
                .variants
                .iter()
                .filter(|variant| !variant.decode_ignore)
                .map(|variant| variant.field_type)
                .collect();

            let impl_generics = build_decode_impl_generics(generics);
            let (_, ty_generics, where_clause) = generics.split_for_impl();

            let expanded = quote! {
                impl #impl_generics ::wayk_proto::serialization::Decode<'dec> for #ty #ty_generics #where_clause {
                    fn decode_from(cursor: &mut ::wayk_proto::io::Cursor<'dec>) -> ::core::result::Result<Self, ::wayk_proto::error::ProtoError> {
                        use ::wayk_proto::error::{ProtoError, ProtoErrorResultExt as _, ProtoErrorKind};
                        use ::wayk_proto::serialization::Encode;

                        let subtype = <#subtype_enum_ty as ::wayk_proto::serialization::Decode>::decode_from(cursor)
                            .chain(ProtoErrorKind::Decoding(stringify!(#ty)))
                            .or_desc("couldn't decode subtype")?;
                        cursor.rewind(subtype.encoded_len());

                        match subtype {
                            #(
                                #subtype_enum_ty::#variants => <#variants_field_ty as ::wayk_proto::serialization::Decode>::decode_from(cursor)
                                    .map(Self::#variants)
                                    .chain(ProtoErrorKind::Decoding(stringify!(#ty)))
                                    .or_desc(concat!(
                                        "couldn't decode ",
                                        stringify!(#ty),
                                        " for subtype ",
                                        stringify!(#variants)
                                    )),
                            )*
                            _ => cursor.peek_rest()
                                .map_err(ProtoError::from)
                                .chain(ProtoErrorKind::Encoding(stringify!(#ty)))
                                .or_desc("couldn't decode custom message")
                                .map(Self::#fallback_variant_ident),
                        }
                    }
                }
            };

            expanded.into()
        }
        parsed::Type::EnumWithFallback(data) => {
            let ty = data.name;
            let underlying_repr = data.underlying_repr;
            let variants = data.variants;
            let fallback_variant = data.fallback_variant;

            let idents: Vec<&Ident> = variants.iter().map(|variant| variant.ident).collect();
            let values: Vec<&LitInt> = variants.iter().map(|variant| &variant.value).collect();

            let expanded = quote! {
                impl ::wayk_proto::serialization::Decode<'_> for #ty {
                    fn decode_from(
                        cursor: &mut ::wayk_proto::io::Cursor<'_>,
                    ) -> ::core::result::Result<Self, ::wayk_proto::error::ProtoError> {
                        let v = #underlying_repr::decode_from(cursor)?;
                        Ok(#ty::from(v))
                    }
                }

                impl ::core::convert::From<#underlying_repr> for #ty {
                    fn from(
                        v: #underlying_repr,
                    ) -> Self {
                        match v {
                            #(
                                #values => #ty::#idents,
                            )*
                            _ => #ty::#fallback_variant(v),
                        }
                    }
                }
            };

            expanded.into()
        }
    }
}

fn find_attr<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a Attribute> {
    attrs
        .iter()
        .find(|attr| attr.path.segments.iter().any(|seg| seg.ident == name))
}

fn impl_trait<F>(ast: &syn::DeriveInput, implementor: F) -> TokenStream
where
    F: FnOnce(parsed::Type<'_>) -> TokenStream,
{
    let ty = &ast.ident;
    let generics = &ast.generics;
    let enc_dec_type = match &ast.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                let fields = fields
                    .named
                    .iter()
                    .map(|field| parsed::Field {
                        decode_ignore: find_attr(&field.attrs, "decode_ignore").is_some(),
                        encode_ignore: find_attr(&field.attrs, "encode_ignore").is_some(),
                        name: field.ident.as_ref().unwrap(),
                        ty: &field.ty,
                    })
                    .collect();

                parsed::Type::Struct(parsed::Struct {
                    name: ty,
                    generics,
                    fields,
                })
            } else {
                unimplemented!("currently only named fields are supported");
            }
        }
        Data::Enum(data) => {
            let meta_enum_attr = find_attr(&ast.attrs, "meta_enum");
            if let Some(meta_enum_attr) = meta_enum_attr {
                let meta = meta_enum_attr
                    .parse_meta()
                    .expect("failed to parse `meta_enum` argument");

                let variants = data
                    .variants
                    .iter()
                    .filter_map(|v| {
                        if find_attr(&v.attrs, "fallback").is_some() {
                            return None;
                        }

                        let field_type = match &v.fields {
                            Fields::Unnamed(fields) => &fields.unnamed.first().unwrap().ty,
                            Fields::Named(_) => panic!("named fields unsupported"),
                            Fields::Unit => panic!("unexpected unit field"),
                        };

                        Some(parsed::MetaEnumVariant {
                            decode_ignore: find_attr(&v.attrs, "decode_ignore").is_some(),
                            encode_ignore: find_attr(&v.attrs, "encode_ignore").is_some(),
                            name: &v.ident,
                            field_type,
                        })
                    })
                    .collect();

                let fallback_variant_ident = data
                    .variants
                    .iter()
                    .find(|v| find_attr(&v.attrs, "fallback").is_some())
                    .map(|v| match &v.fields {
                        Fields::Unnamed(_) => &v.ident,
                        Fields::Named(_) => panic!("unexpected named field"),
                        Fields::Unit => panic!("unexpected unit field"),
                    })
                    .expect("fallback variant missing");

                parsed::Type::MetaEnum(parsed::MetaEnum {
                    name: ty,
                    generics,
                    meta,
                    variants,
                    fallback_variant_ident,
                })
            } else {
                let variants = data
                    .variants
                    .iter()
                    .filter_map(|variant| {
                        let attr = find_attr(&variant.attrs, "value")?;
                        let meta = attr.parse_meta().expect("failed to parse `value` attribute");
                        let lit_int = if let Meta::NameValue(name) = meta {
                            if let Lit::Int(lit_int) = name.lit {
                                lit_int
                            } else {
                                panic!("wrong literal in `value` attribute parameter. Expected a int literal.");
                            }
                        } else {
                            panic!(r#"wrong meta for `value`. Expected a name value (eg: value = 1)."#);
                        };

                        Some(parsed::VariantWithValue {
                            ident: &variant.ident,
                            value: lit_int,
                        })
                    })
                    .collect();

                let fallback_variant = data
                    .variants
                    .iter()
                    .find(|v| find_attr(&v.attrs, "fallback").is_some())
                    .expect("fallback variant not found");

                parsed::Type::EnumWithFallback(parsed::EnumWithFallback {
                    name: ty,
                    underlying_repr: match &fallback_variant.fields {
                        Fields::Unnamed(field) => &field.unnamed.first().unwrap().ty,
                        Fields::Named(_) => panic!("named fields unsupported"),
                        Fields::Unit => panic!("unexpected unit field"),
                    },
                    variants,
                    fallback_variant: &fallback_variant.ident,
                })
            }
        }
        Data::Union(_) => unimplemented!("union"),
    };

    implementor(enc_dec_type)
}
