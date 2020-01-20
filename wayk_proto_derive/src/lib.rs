#![no_std]

extern crate alloc;
extern crate proc_macro;
extern crate proc_macro2;

use alloc::vec::Vec;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    punctuated::Punctuated, token::Add, Attribute, Data, Fields, Generics, Ident, Lifetime, LifetimeDef, Lit, Meta,
    Type,
};

mod parsed {
    use alloc::vec::Vec;

    pub enum Type<'a> {
        Struct(Struct<'a>),
        FieldlessEnum(FieldlessEnum<'a>),
        MetaEnum(MetaEnum<'a>),
    }

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

    pub struct FieldlessEnum<'a> {
        pub name: &'a syn::Ident,
        pub underlying_repr: syn::Ident,
    }

    pub struct MetaEnum<'a> {
        pub name: &'a syn::Ident,
        pub generics: &'a syn::Generics,
        pub subtype_enum_ty: syn::Ident,
        pub meta_variants: Vec<MetaVariant<'a>>,
    }

    pub struct MetaVariant<'a> {
        pub decode_ignore: bool,
        pub encode_ignore: bool,
        pub name: &'a syn::Ident,
        pub field_type: &'a syn::Type,
    }
}

#[proc_macro_derive(Encode, attributes(meta_enum, encode_ignore))]
pub fn encode_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("failed to parse input");
    impl_trait(&ast, impl_encode)
}

fn impl_encode(ty: parsed::Type<'_>) -> TokenStream {
    match ty {
        parsed::Type::Struct(data) => {
            let ty = data.name;
            let (impl_generics, ty_generics, where_clause) = data.generics.split_for_impl();
            let fields = data
                .fields
                .iter()
                .filter(|field| !field.encode_ignore)
                .map(|field| field.name)
                .collect::<Vec<&Ident>>();

            let expanded = quote! {
                impl #impl_generics ::wayk_proto::serialization::Encode for #ty #ty_generics #where_clause {
                    fn encoded_len(&self) -> usize {
                        #(
                            self.#fields.encoded_len()
                        )+*
                    }

                    fn encode_into<W: ::std::io::Write>(&self, writer: &mut W) -> ::core::result::Result<(), ::wayk_proto::error::ProtoError> {
                        use ::wayk_proto::error::{ProtoErrorKind, ProtoErrorResultExt};
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

            let variants: Vec<&Ident> = data
                .meta_variants
                .iter()
                .filter(|variant| !variant.encode_ignore)
                .map(|variant| variant.name)
                .collect();

            let expanded = quote! {
                impl #impl_generics ::wayk_proto::serialization::Encode for #ty #ty_generics #where_clause {
                    fn encoded_len(&self) -> usize {
                        match self {
                            #(
                                Self::#variants(msg) => msg.encoded_len(),
                            )*
                        }
                    }

                    fn encode_into<W: ::std::io::Write>(&self, writer: &mut W) -> ::core::result::Result<(), ::wayk_proto::error::ProtoError> {
                        use ::wayk_proto::error::{ProtoErrorKind, ProtoErrorResultExt};
                        match self {
                            #(
                                Self::#variants(msg) => msg
                                    .encode_into(writer)
                                    .chain(ProtoErrorKind::Encoding(stringify!(#ty)))
                                    .or_desc(concat!("couldn't encode ", stringify!(#variants)," message")),
                            )*
                        }
                    }
                }
            };

            expanded.into()
        }
        parsed::Type::FieldlessEnum(data) => {
            let ty = data.name;
            let underlying_repr = data.underlying_repr;

            let expanded = quote! {
                impl ::wayk_proto::serialization::Encode for #ty {
                    fn encoded_len(&self) -> usize {
                        ::core::mem::size_of::<#underlying_repr>()
                    }

                    fn encode_into<W: ::std::io::Write>(
                        &self,
                        writer: &mut W,
                    ) -> ::core::result::Result<(), ::wayk_proto::error::ProtoError> {
                        <#underlying_repr>::encode_into(&(*self as #underlying_repr), writer)
                    }
                }

                impl #ty {
                    fn to_primitive(&self) -> #underlying_repr {
                        *self as #underlying_repr
                    }
                }
            };

            expanded.into()
        }
    }
}

#[proc_macro_derive(Decode, attributes(meta_enum, decode_ignore))]
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
                    fn decode_from(cursor: &mut ::std::io::Cursor<&'dec [u8]>) -> ::core::result::Result<Self, ::wayk_proto::error::ProtoError> {
                        use ::wayk_proto::error::{ProtoErrorResultExt, ProtoErrorKind};
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
            let subtype_enum_ty = &data.subtype_enum_ty;

            let variants: Vec<&Ident> = data
                .meta_variants
                .iter()
                .filter(|variant| !variant.decode_ignore)
                .map(|variant| variant.name)
                .collect();
            let variants_field_ty: Vec<&Type> = data
                .meta_variants
                .iter()
                .filter(|variant| !variant.decode_ignore)
                .map(|variant| variant.field_type)
                .collect();

            let impl_generics = build_decode_impl_generics(generics);
            let (_, ty_generics, where_clause) = generics.split_for_impl();

            let expanded = quote! {
                impl #impl_generics ::wayk_proto::serialization::Decode<'dec> for #ty #ty_generics #where_clause {
                    fn decode_from(cursor: &mut ::std::io::Cursor<&'dec [u8]>) -> ::core::result::Result<Self, ::wayk_proto::error::ProtoError> {
                        use ::wayk_proto::error::{ProtoErrorResultExt, ProtoErrorKind};
                        use ::wayk_proto::serialization::Encode;
                        use ::std::io::{Seek, SeekFrom};

                        let subtype = <#subtype_enum_ty as ::wayk_proto::serialization::Decode>::decode_from(cursor)
                            .chain(ProtoErrorKind::Decoding(stringify!(#ty)))
                            .or_desc("couldn't decode subtype")?;
                        cursor.seek(SeekFrom::Current(-(subtype.encoded_len() as i64)))
                            .expect("seek back after subtype decoding failed"); // cannot fail

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
                        }
                    }
                }
            };

            expanded.into()
        }
        parsed::Type::FieldlessEnum(data) => {
            let ty = data.name;
            let underlying_repr = data.underlying_repr;

            let from_primitive = Ident::new(&alloc::format!("from_{}", underlying_repr), Span::call_site());

            let expanded = quote! {
                impl ::wayk_proto::serialization::Decode<'_> for #ty {
                    fn decode_from(
                        cursor: &mut ::std::io::Cursor<&[u8]>,
                    ) -> ::core::result::Result<Self, ::wayk_proto::error::ProtoError> {
                        use ::wayk_proto::error::{ProtoErrorKind, ProtoErrorResultExt};
                        let v = #underlying_repr::decode_from(cursor)?;
                        ::num::FromPrimitive::#from_primitive(v)
                            .chain(ProtoErrorKind::Decoding(stringify!($ty)))
                            .or_else_desc(||
                                format!(concat!("no variant in ", stringify!(#ty), " for value {}"), v)
                            )
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
            let repr_attr = find_attr(&ast.attrs, "repr");
            if let Some(meta_enum_attr) = meta_enum_attr {
                let meta = meta_enum_attr
                    .parse_meta()
                    .expect("failed to parse `meta_enum` argument");
                let subtype_enum_ty = if let Meta::NameValue(name) = meta {
                    if let Lit::Str(s) = name.lit {
                        Ident::new(&s.value(), Span::call_site())
                    } else {
                        panic!("wrong literal in `meta_enum` attribute parameter. Expected a string literal for the subtype enum.");
                    }
                } else {
                    panic!(r#"wrong meta for `meta_enum`. Expected a name value (eg: meta_enum = "...")."#);
                };

                let mut meta_variants = Vec::new();
                for variant in &data.variants {
                    let variant = parsed::MetaVariant {
                        decode_ignore: find_attr(&variant.attrs, "decode_ignore").is_some(),
                        encode_ignore: find_attr(&variant.attrs, "encode_ignore").is_some(),
                        name: &variant.ident,
                        field_type: match &variant.fields {
                            Fields::Unnamed(field) => &field.unnamed.first().unwrap().ty,
                            Fields::Named(_) => panic!("named fields unsupported"),
                            Fields::Unit => panic!("unexpected unit field"),
                        },
                    };

                    meta_variants.push(variant);
                }

                parsed::Type::MetaEnum(parsed::MetaEnum {
                    name: ty,
                    generics,
                    subtype_enum_ty,
                    meta_variants,
                })
            } else if let Some(repr_attr) = repr_attr {
                parsed::Type::FieldlessEnum(parsed::FieldlessEnum {
                    name: ty,
                    underlying_repr: repr_attr.parse_args().expect("couldn't parse repr type"),
                })
            } else {
                panic!("meta_enum or repr attribute missing")
            }
        }
        Data::Union(_) => unimplemented!("union"),
    };

    implementor(enc_dec_type)
}
