//! Dynomite-derive provides procedural macros for deriving dynomite types
//! for your structs
//!
//! # Examples
//!
//! ```ignore
//! use dynomite::{Item, FromAttributes, Attributes};
//! use dynomite::dynamodb::AttributeValue;
//!
//! // derive Item
//! #[derive(Item, PartialEq, Debug, Clone)]
//! struct Person {
//!   #[dynomite(partition_key)] id: String
//! }
//!
//!   let person = Person { id: "123".into() };
//!   // convert person to string keys and attribute values
//!   let attributes: Attributes = person.clone().into();
//!   // convert attributes into person type
//!   assert_eq!(person, Person::from_attrs(attributes).unwrap());
//!
//!   // dynamodb types require only primary key attributes and may contain
//!   // other fields. when looking up items only those key attributes are required
//!   // dynomite derives a new {Name}Key struct for your which contains
//!   // only those and also implements Item
//!   let key = PersonKey { id: "123".into() };
//!   let key_attributes: Attributes = key.clone().into();
//!   // convert attributes into person type
//!   assert_eq!(key, PersonKey::from_attrs(key_attributes).unwrap());
//! ```

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    Data::{Enum, Struct},
    DataStruct, DeriveInput, Field, Fields, Ident, Meta, Variant, Visibility,
};

/// Derives `dynomite::Item` type for struts with named fields
///
/// # Attributes
///
/// * `#[dynomite(partition_key)]` - required attribute, expected to be applied the target [partition attribute](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.CoreComponents.html#HowItWorks.CoreComponents.PrimaryKey) field with an derivable DynamoDB attribute value of String, Number or Binary
/// * `#[dynomite(sort_key)]` - optional attribute, may be applied to one target [sort attribute](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.CoreComponents.html#HowItWorks.CoreComponents.SecondaryIndexes) field with an derivable DynamoDB attribute value of String, Number or Binary
/// * `#[dynomite(rename = "actualName")]` - optional attribute, may be applied any item attribute field, useful when the DynamoDB table you're interfacing with has attributes whose names don't following Rust's naming conventions
///
/// # Panics
///
/// This proc macro will panic when applied to other types
#[proc_macro_derive(Item, attributes(partition_key, sort_key, dynomite))]
pub fn derive_item(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input);

    let gen = match expand_item(ast) {
        Ok(g) => g,
        Err(e) => return e.to_compile_error().into(),
    };

    gen.into_token_stream().into()
}

/// Derives `dynomite::Attribute` for enum types
///
/// # Panics
///
/// This proc macro will panic when applied to other types
#[proc_macro_derive(Attribute)]
pub fn derive_attribute(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input);
    let gen = expand_attribute(ast);
    gen.into_token_stream().into()
}

fn expand_attribute(ast: DeriveInput) -> impl ToTokens {
    let name = &ast.ident;
    match ast.data {
        Enum(variants) => {
            make_dynomite_attr(name, &variants.variants.into_iter().collect::<Vec<_>>())
        }
        _ => panic!("Dynomite Attributes can only be generated for enum types"),
    }
}

/// ```rust,ignore
/// impl ::dynomite::Attribute for Name {
///   fn into_attr(self) -> ::dynomite::dynamodb::AttributeValue {
///     let arm = match self {
///        Name::Variant => "Variant".to_string()
///     };
///     ::dynomite::dynamodb::AttributeValue {
///        s: Some(arm),
///        ..Default::default()
///     }
///   }
///   fn from_attr(value: ::dynomite::dynamodb::AttributeValue) -> Result<Self, ::dynomite::AttributeError> {
///     value.s.ok_or(::dynomite::AttributeError::InvalidType)
///       .and_then(|value| match &value[..] {
///          "Variant" => Ok(Name::Variant),
///          _ => Err(::dynomite::AttributeError::InvalidFormat)
///       })
///   }
/// }
/// ```
fn make_dynomite_attr(
    name: &Ident,
    variants: &[Variant],
) -> impl ToTokens {
    let attr = quote!(::dynomite::Attribute);
    let err = quote!(::dynomite::AttributeError);
    let into_match_arms = variants.iter().map(|var| {
        let vname = &var.ident;
        quote! {
            #name::#vname => stringify!(#vname).to_string(),
        }
    });
    let from_match_arms = variants.iter().map(|var| {
        let vname = &var.ident;
        quote! {
            stringify!(#vname) => ::std::result::Result::Ok(#name::#vname),
        }
    });

    quote! {
        impl #attr for #name {
            fn into_attr(self) -> ::dynomite::dynamodb::AttributeValue {
                let arm = match self {
                    #(#into_match_arms)*
                };
                ::dynomite::dynamodb::AttributeValue {
                    s: ::std::option::Option::Some(arm),
                    ..::std::default::Default::default()
                }
            }
            fn from_attr(value: ::dynomite::dynamodb::AttributeValue) -> ::std::result::Result<Self, #err> {
                value.s.ok_or(::dynomite::AttributeError::InvalidType)
                    .and_then(|value| match &value[..] {
                        #(#from_match_arms)*
                        _ => ::std::result::Result::Err(::dynomite::AttributeError::InvalidFormat)
                    })
            }
        }
    }
}

fn expand_item(ast: DeriveInput) -> syn::Result<impl ToTokens> {
    let name = &ast.ident;
    let vis = &ast.vis;
    match ast.data {
        Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(named) => {
                make_dynomite_item(vis, name, &named.named.into_iter().collect::<Vec<_>>())
            }
            _ => panic!("Dynomite Items require named fields"),
        },
        _ => panic!("Dynomite Items can only be generated for structs"),
    }
}

fn make_dynomite_item(
    vis: &Visibility,
    name: &Ident,
    fields: &[Field],
) -> syn::Result<impl ToTokens> {
    let dynamodb_traits = get_dynomite_item_traits(vis, name, fields)?;
    let from_attribute_map = get_from_attributes_trait(name, fields)?;
    let to_attribute_map = get_to_attribute_map_trait(name, fields)?;

    Ok(quote! {
        #from_attribute_map
        #to_attribute_map
        #dynamodb_traits
    })
}

fn get_to_attribute_map_trait(
    name: &Ident,
    fields: &[Field],
) -> syn::Result<impl ToTokens> {
    let attributes = quote!(::dynomite::Attributes);
    let from = quote!(::std::convert::From);
    let to_attribute_map = get_to_attribute_map_function(name, fields)?;

    Ok(quote! {
        impl #from<#name> for #attributes {
            #to_attribute_map
        }
    })
}

/// Get the items in `attributes` with only a single path segment with an
/// ident of `dynomite`.
fn dynomite_attributes(attributes: &[syn::Attribute]) -> impl Iterator<Item = &'_ syn::Attribute> {
    attributes
        .iter()
        .filter(|attr| attr.path.segments.len() == 1 && attr.path.segments[0].ident == "dynomite")
}

/// Get `Ok` from `#[dynomite(name = value)]` if applicable, otherwise `Err`.
fn get_name_eq_value_attribute_lit(
    attribute: &syn::Attribute,
    name: &'_ str,
) -> syn::Result<syn::Lit> {
    use syn::spanned::Spanned as _;

    // #[dynomite()]
    let mut tokens = match attribute.tokens.clone().into_iter().next() {
        Some(proc_macro2::TokenTree::Group(g)) => g.stream().into_iter(),
        _ => {
            return Err(syn::Error::new(
                attribute.span(),
                format!("expected form `#[dynomite({} = value)]`", name),
            ))
        }
    };

    // #[dynomite(name)]
    match tokens.next() {
        Some(proc_macro2::TokenTree::Ident(ref ident)) if *ident == name => {}
        Some(other) => {
            return Err(syn::Error::new(
                other.span(),
                format!("expected `{}`", name),
            ))
        }
        None => {
            return Err(syn::Error::new(
                attribute.span(),
                format!("expected form `#[dynomite({} = value)]`", name),
            ))
        }
    };

    // #[dynomite(name = )]
    match tokens.next() {
        Some(proc_macro2::TokenTree::Punct(ref punct)) if punct.as_char() == '=' => {}
        Some(other) => return Err(syn::Error::new(other.span(), "expected `=`")),
        None => {
            return Err(syn::Error::new(
                attribute.span(),
                format!("expected form `#[dynomite({} = value)]`", name),
            ))
        }
    };

    // #[dynomite(name = value)]
    let lit = match tokens.next() {
        Some(proc_macro2::TokenTree::Literal(lit)) => Ok(syn::Lit::new(lit)),
        Some(other) => Err(syn::Error::new(
            other.span(),
            "expected value to be a literal",
        )),
        None => {
            return Err(syn::Error::new(
                attribute.span(),
                format!("expected form `#[dynomite({} = value)]`", name),
            ))
        }
    };

    // Make sure there are no more tokens
    if let Some(token) = tokens.next() {
        return Err(syn::Error::new(
            token.span(),
            format!("expected form `#[dynomite({} = value)]`", name),
        ));
    }

    lit
}

/// The name of the field to be used during de/serialization
///
/// # Returns
/// `Ok("foo")` from `#[dynomite(rename = "foo")]` if applicable, otherwise `Ok(field.ident)`
/// - `Err` if multiple `#[dynomite(rename = "foo")]` attributes are present on `field`
/// - `Err` if `value` in `#[dynomite(rename = value)]` is not a string literal
/// - `Err` if `value` in `#[dynomite(rename = value)]` is an empty string literal
fn get_field_deser_name(field: &Field) -> syn::Result<String> {
    use syn::spanned::Spanned as _;

    let rename_value_opt = {
        let rename_value_lits: Vec<_> = dynomite_attributes(&field.attrs)
            .filter_map(|attr| get_name_eq_value_attribute_lit(attr, "rename").ok())
            .collect();

        if rename_value_lits.len() > 1 {
            // Pick the 2nd since it is the first duplicate
            // TODO: get the attr's span, not just the lit's
            let lit_to_err_on = &rename_value_lits[1];
            return Err(syn::Error::new(
                lit_to_err_on.span(),
                "fields may have a maximum of 1 `#[dynomite(rename = \"...\")]` attribute",
            ));
        }

        match rename_value_lits.get(0) {
            Some(syn::Lit::Str(lit_str)) => {
                let value = lit_str.value();

                if value.trim().is_empty() {
                    return Err(syn::Error::new(
                        lit_str.span(),
                        "expected non-empty string literal value in `#[dynomite(rename = \"...\")]` attribute",
                    ));
                }

                value.into()
            }
            Some(other) => {
                return Err(syn::Error::new(
                    other.span(),
                    "expected string literal value in `#[dynomite(rename = ...)]` attribute",
                ));
            }
            _ => None,
        }
    };

    Ok(rename_value_opt.unwrap_or_else(|| {
        field
            .ident
            .as_ref()
            .expect("should have an identifier")
            .to_string()
    }))
}

fn get_to_attribute_map_function(
    name: &Ident,
    fields: &[Field],
) -> syn::Result<impl ToTokens> {
    let to_attribute_value = quote!(::dynomite::Attribute::into_attr);

    let field_conversions = fields
        .iter()
        .map(|field| {
            let field_deser_name = &match get_field_deser_name(field) {
                Ok(name) => name,
                Err(e) => return Err(e),
            };

            let field_ident = &field.ident;
            Ok(quote! {
                values.insert(
                    #field_deser_name.to_string(),
                    #to_attribute_value(item.#field_ident)
                );
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        fn from(item: #name) -> Self {
            let mut values = Self::new();
            #(#field_conversions)*
            values
        }
    })
}

/// ```rust,ignore
/// impl ::dynomite::FromAttributes for Name {
///   fn from_attrs(mut item: ::dynomite::Attributes) -> Result<Self, ::dynomite::Error> {
///     Ok(Self {
///        field_name: ::dynomite::Attribute::from_attr(
///           item.remove("field_deser_name").ok_or(Error::MissingField { name: "field_deser_name".into() })?
///        )
///      })
///   }
/// }
/// ```
fn get_from_attributes_trait(
    name: &Ident,
    fields: &[Field],
) -> syn::Result<impl ToTokens> {
    let from_attrs = quote!(::dynomite::FromAttributes);
    let from_attribute_map = get_from_attributes_function(fields)?;

    Ok(quote! {
        impl #from_attrs for #name {
            #from_attribute_map
        }
    })
}

fn get_from_attributes_function(fields: &[Field]) -> syn::Result<impl ToTokens> {
    let attributes = quote!(::dynomite::Attributes);
    let from_attribute_value = quote!(::dynomite::Attribute::from_attr);
    let err = quote!(::dynomite::AttributeError);

    let field_conversions = fields.iter().map(|field| {
        let field_deser_name = &match get_field_deser_name(field) {
            Ok(name) => name,
            Err(e) => return Err(e),
        };

        let field_ident = &field.ident;
        Ok(quote! {
            #field_ident: #from_attribute_value(
                attrs.remove(#field_deser_name)
                    .ok_or(::dynomite::AttributeError::MissingField { name: #field_deser_name.to_string() })?
            )?
        })
    }).collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        fn from_attrs(mut attrs: #attributes) -> ::std::result::Result<Self, #err> {
            ::std::result::Result::Ok(Self {
                #(#field_conversions),*
            })
        }
    })
}

fn get_dynomite_item_traits(
    vis: &Visibility,
    name: &Ident,
    fields: &[Field],
) -> syn::Result<impl ToTokens> {
    let impls = get_item_impls(vis, name, fields)?;

    Ok(quote! {
        #impls
    })
}

fn get_item_impls(
    vis: &Visibility,
    name: &Ident,
    fields: &[Field],
) -> syn::Result<impl ToTokens> {
    let item_trait = get_item_trait(name, fields)?;
    let key_struct = get_key_struct(vis, name, fields)?;

    Ok(quote! {
        #item_trait
        #key_struct
    })
}

/// ```rust,ignore
/// impl ::dynomite::Item for Name {
///   fn key(&self) -> ::std::collections::HashMap<String, ::dynomite::dynamodb::AttributeValue> {
///     let mut keys = ::std::collections::HashMap::new();
///     keys.insert("field_deser_name", to_attribute_value(field));
///     keys
///   }
/// }
/// ```
fn get_item_trait(
    name: &Ident,
    fields: &[Field],
) -> syn::Result<impl ToTokens> {
    let item = quote!(::dynomite::Item);
    let attribute_map = quote!(
        ::std::collections::HashMap<String, ::dynomite::dynamodb::AttributeValue>
    );
    let partition_key_field = field_with_attribute(&fields, "partition_key");
    let sort_key_field = field_with_attribute(&fields, "sort_key");

    let partition_key_insert = partition_key_field
        .as_ref()
        .map(get_key_inserter)
        .transpose()?;

    let sort_key_insert = sort_key_field.as_ref().map(get_key_inserter).transpose()?;

    Ok(partition_key_field
        .map(|_| {
            quote! {
                impl #item for #name {
                    fn key(&self) -> #attribute_map {
                        let mut keys = ::std::collections::HashMap::new();
                        #partition_key_insert
                        #sort_key_insert
                        keys
                    }
                }
            }
        })
        .unwrap_or_else(proc_macro2::TokenStream::new))
}

/// Find a single `Field` from `fields` that has an attribute of the form
/// `#[dynomite(attribute_name)]`.
///
/// # Panics
/// - if there are multiple fields, in `fields`, with an attribute of the form
///   `#[dynomite(attribute_name)]`
fn field_with_attribute(
    fields: &[Field],
    attribute_name: &str,
) -> Option<Field> {
    let mut fields = fields.iter().cloned().filter(|field| {
        field.attrs.iter().any(|attr| match attr.parse_meta() {
            Ok(Meta::List(syn::MetaList { path, nested, .. })) => {
                if path.segments.len() > 1 {
                    return false;
                }

                if path.segments[0].ident != "dynomite" {
                    return false;
                }

                match nested.first() {
                    Some(syn::NestedMeta::Meta(syn::Meta::Path(path))) => {
                        if path.segments.len() > 1 {
                            return false;
                        }

                        path.segments[0].ident == attribute_name
                    }
                    _ => false,
                }
            }
            _ => false,
        })
    });
    let field = fields.next();
    if fields.next().is_some() {
        panic!("Can't set more than one `{}`", attribute_name);
    }
    field
}

/// ```rust,ignore
/// keys.insert(
///   "field_deser_name", to_attribute_value(field)
/// );
/// ```
fn get_key_inserter(field: &Field) -> syn::Result<impl ToTokens> {
    let to_attribute_value = quote!(::dynomite::Attribute::into_attr);
    let field_deser_name = &get_field_deser_name(field)?;
    let field_ident = &field.ident;
    Ok(quote! {
        keys.insert(
            #field_deser_name.to_string(),
            #to_attribute_value(self.#field_ident.clone())
        );
    })
}

/// ```rust,ignore
/// #[derive(Item, Debug, Clone, PartialEq)]
/// pub struct NameKey {
///    partition_key_field,
///    range_key
/// }
/// ```
fn get_key_struct(
    vis: &Visibility,
    name: &Ident,
    fields: &[Field],
) -> syn::Result<impl ToTokens> {
    let name = Ident::new(&format!("{}Key", name), Span::call_site());

    let partition_key_field = field_with_attribute(&fields, "partition_key")
        .map(|mut field| {
            // rename the field to the de/ser name
            if let Err(e) = rename_field_to_deser_name(&mut field) {
                return Err(e);
            }

            // remove attributes (because key structs don't need attrs) but
            // _after_ renaming the field so `get_field_deser_name` still works
            field.attrs = vec![];

            Ok(quote! {
                #field
            })
        })
        .transpose()?;

    let sort_key_field = field_with_attribute(&fields, "sort_key")
        .map(|mut field| {
            // rename the field to the de/ser name
            if let Err(e) = rename_field_to_deser_name(&mut field) {
                return Err(e);
            }

            // remove attributes (because key structs don't need attrs) but
            // _after_ renaming the field so `get_field_deser_name` still works
            field.attrs = vec![];

            Ok(quote! {
                #field
            })
        })
        .transpose()?
        .unwrap_or_else(proc_macro2::TokenStream::new);

    Ok(partition_key_field
        .map(|partition_key_field| {
            quote! {
                #[derive(Item, Debug, Clone, PartialEq)]
                #vis struct #name {
                    #partition_key_field,
                    #sort_key_field
                }
            }
        })
        .unwrap_or_else(proc_macro2::TokenStream::new))
}

/// Change `field.ident` to the value returned by `get_field_deser_name`
fn rename_field_to_deser_name(field: &mut Field) -> syn::Result<()> {
    let field_deser_name = get_field_deser_name(field)?;

    field.ident = field
        .ident
        .as_ref()
        .map(|ident| syn::Ident::new(&field_deser_name, ident.span()));

    Ok(())
}
