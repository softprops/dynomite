//! dynomite Item type derivation for structs
//!
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::{DeriveInput, Field, Ident, Visibility};
use syn::Body::Struct;
use syn::VariantData::Struct as StructData;

#[proc_macro_derive(Item, attributes(hash, range))]
pub fn dynomite_item(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = expand(&ast);
    gen.parse().unwrap()
}

fn expand(ast: &DeriveInput) -> Tokens {
    let name = &ast.ident;
    let vis = &ast.vis;
    match ast.body {
        Struct(StructData(ref fields)) => make_dynomite_item(vis, name, fields),
        _ => panic!("Dynomite Items can only be generated for structs"),
    }
}

fn make_dynomite_item(vis: &Visibility, name: &Ident, fields: &[Field]) -> Tokens {
    let dynamodb_traits = get_dynomite_traits(vis, name, fields);
    let to_attribute_map = get_from_attributes_trait(name, fields);
    let from_attribute_map = get_to_attribute_map_trait(name, fields);

    quote! {
        #from_attribute_map
        #to_attribute_map
        #dynamodb_traits
    }
}

fn get_to_attribute_map_trait(name: &Ident, fields: &[Field]) -> Tokens {
    let attribute_map = quote!(
        ::std::collections::HashMap<String, ::rusoto_dynamodb::AttributeValue>
    );
    let from = quote!(::std::convert::From);
    let to_attribute_map = get_to_attribute_map_function(name, fields);

    quote! {
        impl #from<#name> for #attribute_map {
            #to_attribute_map
        }
    }
}

fn get_to_attribute_map_function(name: &Ident, fields: &[Field]) -> Tokens {
    let to_attribute_value = quote!(::dynomite::Attribute::into_attr);

    let field_conversions = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            values.insert(
                stringify!(#field_name).to_string(),
                #to_attribute_value(item.#field_name)
            );
        }
    });

    quote! {
        fn from(item: #name) -> Self {
            let mut values = Self::new();
            #(#field_conversions)*
            values
        }
    }
}

///
/// impl ::dynomite::FromAttributeValues for Name {
///   fn from_attrs(mut item: ::std::collections::HashMap<String, ::rusoto_dynamodb::AttributeValue>) -> Result<Self, String> {
///     Ok(Self {
///        field_name: ::dynomite::Attribute::from_attr(
///           item.remove("field_name").ok_or("missing".to_string())?
///        )
///      })
///   }
/// }
fn get_from_attributes_trait(name: &Ident, fields: &[Field]) -> Tokens {
    let attribute_map = quote!(
        ::std::collections::HashMap<String, ::rusoto_dynamodb::AttributeValue>
    );
    let from_attrs = quote!(::dynomite::FromAttributeValues);
    let from_attribute_map = get_from_attributes_function(fields);

    quote! {
        impl #from_attrs for #name {
            #from_attribute_map
        }
    }
}

fn get_from_attributes_function(fields: &[Field]) -> Tokens {
    let attribute_map = quote!(
        ::std::collections::HashMap<String, ::rusoto_dynamodb::AttributeValue>
    );
    let from_attribute_value = quote!(::dynomite::Attribute::from_attr);
    let field_conversions = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: #from_attribute_value(
                item.remove(stringify!(#field_name))
                    .ok_or("missing".to_string())?
            )?
        }
    });

    quote! {
        fn from_attrs(mut item: #attribute_map) -> Result<Self, String> {
            Ok(Self {
                #(#field_conversions),*
            })
        }
    }
}

fn get_dynomite_traits(vis: &Visibility, name: &Ident, fields: &[Field]) -> Tokens {
    let impls = get_impls(vis, name, fields);

    quote! {
        #impls
    }
}

fn get_impls(vis: &Visibility, name: &Ident, fields: &[Field]) -> Tokens {
    let item_trait = get_item_trait(name, fields);
    let key_struct = get_key_struct(vis, name, fields);

    quote! {
        #item_trait
        #key_struct
    }
}

///
/// impl ::dynomite::Item for Name {
///   fn key(&self) -> ::std::collections::HashMap<String, ::rusoto_dynamodb::AttributeValue> {
///     let mut keys = ::std::collections::HashMap::new();
///     keys.insert("field_name", to_attribute_value(field));
///     keys
///   }
/// }
///
fn get_item_trait(name: &Ident, fields: &[Field]) -> Tokens {
    let dynamodb_insertable = quote!(::dynomite::Item);
    let attribute_map = quote!(
        ::std::collections::HashMap<String, ::rusoto_dynamodb::AttributeValue>
    );
    let hash_key_name = get_field_name_with_attribute(&fields, "hash");
    let range_key_name = get_field_name_with_attribute(&fields, "range");

    let hash_key_inserter = get_key_inserter(&hash_key_name);
    let range_key_inserter = get_key_inserter(&range_key_name);

    hash_key_name
        .map(|_| {
            quote!{
                impl #dynamodb_insertable for #name {
                    fn key(&self) -> #attribute_map {
                        let mut keys = ::std::collections::HashMap::new();
                        #hash_key_inserter
                        #range_key_inserter
                        keys
                    }
                }
            }
        })
        .unwrap_or(quote!{})
}

fn get_field_name_with_attribute(fields: &[Field], attribute_name: &str) -> Option<Ident> {
    get_field_with_attribute(fields, attribute_name).map(|field| {
        field
            .ident
            .expect(&format!("{} should have an identifier", attribute_name))
    })
}

fn get_field_with_attribute(fields: &[Field], attribute_name: &str) -> Option<Field> {
    let mut fields = fields
        .iter()
        .cloned()
        .filter(|field| field.attrs.iter().any(|attr| attr.name() == attribute_name));

    let field = fields.next();
    if let Some(_) = fields.next() {
        panic!("Can't set more than one {} key", attribute_name);
    }
    field
}

/// keys.insert(
///   "field_name", to_attribute_value(field)
/// )
fn get_key_inserter(field_name: &Option<Ident>) -> Tokens {
    let to_attribute_value = quote!(::dynomite::Attribute::into_attr);
    field_name
        .as_ref()
        .map(|field_name| {
            quote!{
                keys.insert(
                    stringify!(#field_name).to_string(),
                    #to_attribute_value(self.#field_name.clone())
                );
            }
        })
        .unwrap_or(quote!())
}

/// #[derive](Item, Debug, Clone, PartialEq)
/// pub struct Name {
///    hash_key,
///    range_key
/// }
fn get_key_struct(vis: &Visibility, name: &Ident, fields: &[Field]) -> Tokens {
    let name = Ident::from(format!("{}Key", name));

    let hash_key = get_field_with_attribute(&fields, "hash");
    let range_key = get_field_with_attribute(&fields, "range")
        .map(|mut range_key| {
            range_key.attrs = vec![];
            quote! {#range_key}
        })
        .unwrap_or(quote!());

    hash_key
        .map(|mut hash_key| {
            hash_key.attrs = vec![];
            quote!{
                #[derive(Item, Debug, Clone, PartialEq)]
                #vis struct #name {
                    #hash_key,
                    #range_key
                }
            }
        })
        .unwrap_or(quote!())
}
