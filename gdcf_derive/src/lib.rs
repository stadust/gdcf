#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::{Attribute, Data, DeriveInput, ExprPath, Fields, GenericParam, Ident, Lit, Meta, MetaList, NestedMeta, Type};

#[proc_macro_derive(FromRawObject, attributes(raw_data, raw_type))]
pub fn from_raw_object_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let generated = impl_from_raw_object(&ast);

    generated.into()
}

fn impl_from_raw_object(ast: &DeriveInput) -> Tokens {
    let name = &ast.ident;
    let mut data = Vec::new();
    let mut gen_types: Vec<Type> = Vec::new();

    'outer: for param in &ast.generics.params {
        if let GenericParam::Type(type_param) = param {
            for attr in &type_param.attrs {
                if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "raw_type" {
                    if let Some(meta) = attr.interpret_meta() {
                        if let Meta::List(MetaList { ident, nested, .. }) = meta {
                            if ident == "raw_type" && nested.len() == 1 {
                                if let Some(NestedMeta::Literal(Lit::Str(ls))) = nested.iter().next() {
                                    gen_types
                                        .push(syn::parse_str(&ls.value()).expect("Raw type for generic parameter didn't parse as type"));
                                    continue 'outer
                                } else {
                                    panic!("Invalid raw_type attribute")
                                }
                            } else {
                                panic!("Invalid raw_type attribute")
                            }
                        } else {
                            panic!("Invalid raw_type attribute")
                        }
                    } else {
                        panic!("Invalid raw_type attribute")
                    }
                }
            }

            panic!("Generic type parameter without raw_type attribute: {}", type_param.ident)
        } else {
            panic!("Uhh, no lifetimes, sorry")
        }
    }

    match ast.data {
        Data::Struct(ref structure) =>
            match structure.fields {
                Fields::Named(ref fields) =>
                    for field in fields.named.iter() {
                        let field_name = &field.ident.unwrap();

                        let mut index = None;
                        let mut flatten = false;
                        let mut deserialize_with: Option<String> = None;
                        let mut custom = None;
                        let mut default = DefaultValue::None;

                        for meta_items in field.attrs.iter().filter_map(get_meta_items) {
                            for meta_item in meta_items {
                                match meta_item {
                                    NestedMeta::Meta(Meta::NameValue(ref nv)) if nv.ident == "index" =>
                                        if let Lit::Int(ref intlit) = nv.lit {
                                            if index.is_some() {
                                                panic!("'index' set twice on field {}", field_name)
                                            }

                                            index = Some(intlit.value() as usize)
                                        } else {
                                            panic!("'index' not integer value on field {}", field_name)
                                        },

                                    NestedMeta::Meta(Meta::Word(word)) if word == "flatten" => {
                                        if flatten {
                                            panic!("'flatten' set twice on field {}", field_name)
                                        }

                                        flatten = true
                                    },

                                    NestedMeta::Meta(Meta::NameValue(ref nv)) if nv.ident == "deserialize_with" =>
                                        if let Lit::Str(ref strlit) = nv.lit {
                                            if deserialize_with.is_some() {
                                                panic!("'deserialize_with' set twice on field {}", field_name)
                                            }

                                            deserialize_with = Some(strlit.value())
                                        } else {
                                            panic!("'deserialize_with' not string value on field {}", field_name)
                                        },

                                    NestedMeta::Meta(Meta::NameValue(ref nv)) if nv.ident == "custom" =>
                                        if let Lit::Str(ref strlit) = nv.lit {
                                            if custom.is_some() {
                                                panic!("'custom' set twice on field {}", field_name)
                                            }

                                            custom = Some(strlit.value())
                                        } else {
                                            panic!("'custom' not string value on field {}", field_name)
                                        },

                                    NestedMeta::Meta(Meta::NameValue(ref nv)) if nv.ident == "default" =>
                                        match default {
                                            DefaultValue::None => default = DefaultValue::Literal(nv.lit.clone()),
                                            _ => panic!("'default' set twice on field {}", field_name),
                                        },

                                    NestedMeta::Meta(Meta::Word(word)) if word == "default" =>
                                        match default {
                                            DefaultValue::None => default = DefaultValue::Default,
                                            _ => panic!("'default' set twice on field {}", field_name),
                                        },

                                    _ => panic!("invalid attribute value on field {}", field_name),
                                }
                            }
                        }

                        let mode = if flatten {
                            Mode::Flatten
                        } else if let Some(path) = custom {
                            Mode::Custom(syn::parse_str(&path).unwrap())
                        } else {
                            let idx = index.expect(&format!("no 'index' specified on field {}", field_name));

                            if let Some(path) = deserialize_with {
                                Mode::With(syn::parse_str(&path).unwrap(), idx, default)
                            } else {
                                Mode::Auto(idx, default)
                            }
                        };

                        data.push(FieldData {
                            field_name: field_name.clone(),
                            mode,
                        })
                    },
                _ => panic!("FromRawObject can only be derived for structs with named fields"),
            },
        _ => panic!("FromRawObject can only be derived for structs"),
    }

    let things = data.into_iter().map(|FieldData { field_name, mode }| {
        match mode {
            Mode::Auto(idx, DefaultValue::None) => {
                quote! {
                    #field_name : raw_obj.get(#idx)? // FromStr
                }
            },
            Mode::Auto(idx, DefaultValue::Literal(lit)) => {
                quote! {
                    #field_name : raw_obj.get_or(#idx, #lit)?   // FromStr
                }
            },
            Mode::Auto(idx, DefaultValue::Default) => {
                quote! {
                    #field_name : raw_obj.get_or_default(#idx)?  // Default + FromStr
                }
            },
            Mode::With(path, idx, DefaultValue::None) =>
                quote! {
                    #field_name : raw_obj.get_with(#idx, #path)?
                },
            Mode::With(path, idx, DefaultValue::Literal(lit)) =>
                quote! {
                    #field_name : raw_obj.get_with_or(#idx, #path, #lit)?
                },
            Mode::With(path, idx, DefaultValue::Default) => {
                quote! {
                    #field_name : raw_obj.get_with_or_default(#idx, #path)?  // Default
                }
            },
            Mode::Flatten =>
                quote! {
                    #field_name : TryFrom::try_from(raw_obj)?
                },
            Mode::Custom(path) =>
                quote! {
                    #field_name: #path(raw_obj)?
                },
        }
    });

    let gen_types2 = gen_types.clone();

    quote! {
        impl TryFrom<RawObject> for #name<#(#gen_types,)*> {
            type Error = ValueError;

            fn try_from(raw_obj: RawObject) -> Result<Self, ValueError> {
                Self::try_from(&raw_obj)
            }
        }

        impl<'a> TryFrom<&'a RawObject> for #name<#(#gen_types2,)*> {
            type Error = ValueError;

            fn try_from(raw_obj: &'a RawObject) -> Result<Self, ValueError> {
                Ok(#name {
                    #(#things,)*
                })
            }
        }
    }
}

enum DefaultValue {
    None,
    Default,
    Literal(Lit),
}

enum Mode {
    Custom(ExprPath),
    With(ExprPath, usize, DefaultValue),
    Auto(usize, DefaultValue),
    Flatten,
}

struct FieldData {
    field_name: Ident,
    mode: Mode,
}

fn get_meta_items(attr: &Attribute) -> Option<Vec<NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "raw_data" {
        match attr.interpret_meta() {
            Some(Meta::List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => None,
        }
    } else {
        None
    }
}
