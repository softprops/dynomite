//! Provides procedural macros for deriving dynomite types for your structs and enum types
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
//! let person = Person { id: "123".into() };
//! // convert person to string keys and attribute values
//! let attributes: Attributes = person.clone().into();
//! // convert attributes into person type
//! assert_eq!(person, Person::from_attrs(attributes).unwrap());
//!
//! // dynamodb types require only primary key attributes and may contain
//! // other fields; when looking up items only those key attributes are required
//! // dynomite derives a new {Name}Key struct for your which contains
//! // only those and also implements Item
//! let key = PersonKey { id: "123".into() };
//! let key_attributes: Attributes = key.clone().into();
//! // convert attributes into person type
//! assert_eq!(key, PersonKey::from_attrs(key_attributes).unwrap());
//! ```

mod attr;
use std::collections::HashSet;

use attr::{EnumAttr, EnumAttrKind, FieldAttr, FieldAttrKind, VariantAttr};

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, punctuated::Punctuated, Attribute, DataStruct, DeriveInput, Field, Fields, Ident,
    Token, Visibility,
};

struct Variant {
    inner: syn::Variant,
    attrs: Vec<VariantAttr>,
}

impl Variant {
    fn deser_name(&self) -> String {
        self.attrs
            .iter()
            .find_map(|it| match &it.kind {
                attr::VariantAttrKind::Rename(it) => Some(it.value()),
            })
            .unwrap_or_else(|| self.inner.ident.to_string())
    }
}

struct DataEnum {
    attrs: Vec<EnumAttr>,
    ident: syn::Ident,
    variants: Vec<Variant>,
}

impl DataEnum {
    fn new(
        ident: Ident,
        inner: syn::DataEnum,
        attrs: &[Attribute],
    ) -> Self {
        let me = Self {
            attrs: parse_attrs(&attrs),
            ident,
            variants: inner
                .variants
                .into_iter()
                .map(|inner| {
                    let attrs = parse_attrs(&inner.attrs);
                    Variant { inner, attrs }
                })
                .collect(),
        };

        // Validate that all enum tag values are unique
        let mut unique_names = HashSet::new();
        for variant in &me.variants {
            if let Some(existing) = unique_names.replace(variant.deser_name()) {
                abort!(
                    variant.inner.ident.span(),
                    "Duplicate tag name detected: `{}`", existing;
                    help = "Please ensure that no `rename = \"tag_value\"` \
                    clauses conflict with each other and remaining enum variants' names"
                );
            }
        }
        me
    }

    fn tag_key(&self) -> String {
        self.attrs
            .iter()
            .find_map(|attr| match &attr.kind {
                EnumAttrKind::Tag(lit) => Some(lit.value()),
            })
            .unwrap_or_else(|| {
                abort!(
                    self.ident,
                    "#[derive(Attributes)] for fat enums must have a sibling \
                    #[dynomite(tag = \"key\")] attribute to specify the descriptor field name.";
                    note = "Only internally tagged enums are supported in this version of dynomite."
                )
            })
    }

    fn impl_from_attributes(&self) -> impl ToTokens {
        let match_arms = self.variants.iter().map(|variant| {
            let variant_ident = &variant.inner.ident;
            let expr = match &variant.inner.fields {
                Fields::Named(_record) => Self::unimplemented_record_variants(&variant),
                Fields::Unnamed(tuple) => {
                    Self::expect_single_item_tuple(&tuple, variant_ident);
                    quote! { Self::#variant_ident(::dynomite::FromAttributes::from_attrs(attrs)?) }
                }
                Fields::Unit => quote! { Self::#variant_ident },
            };
            let variant_deser_name = variant.deser_name();
            quote! { #variant_deser_name => #expr, }
        });

        let enum_ident = &self.ident;
        let tag_key = self.tag_key();
        quote! {
            impl ::dynomite::FromAttributes for #enum_ident {
                fn from_attrs(attrs: &mut ::dynomite::Attributes) -> ::std::result::Result<Self, ::dynomite::AttributeError> {
                    use ::std::{string::String, result::Result::{Ok, Err}};
                    use ::dynomite::{Attribute, AttributeError};

                    let tag = attrs.remove(#tag_key).ok_or_else(|| {
                        AttributeError::MissingField {
                            name: #tag_key.to_owned(),
                        }
                    })?;
                    let tag: String = Attribute::from_attr(tag)?;
                    Ok(match tag.as_str() {
                        #(#match_arms)*
                        _ => return Err(AttributeError::InvalidFormat)
                    })
                }
            }
        }
    }

    fn impl_into_attributes(&self) -> impl ToTokens {
        let enum_ident = &self.ident;

        let match_arms = self.variants.iter().map(|variant| {
            let variant_ident = &variant.inner.ident;
            let variant_deser_name = variant.deser_name();
            match &variant.inner.fields {
                Fields::Named(_record) => Self::unimplemented_record_variants(&variant),
                Fields::Unnamed(tuple) => {
                    Self::expect_single_item_tuple(&tuple, variant_ident);

                    quote! {
                        Self::#variant_ident(variant) => {
                            ::dynomite::IntoAttributes::into_attrs(variant, attrs);
                            #variant_deser_name
                        }
                    }
                }
                Fields::Unit => quote! { Self::#variant_ident => #variant_deser_name, },
            }
        });

        let tag_key = self.tag_key();

        quote! {
            impl ::dynomite::IntoAttributes for #enum_ident {
                fn into_attrs(self, attrs: &mut ::dynomite::Attributes) {
                    let tag = match self {
                        #(#match_arms)*
                    };
                    let tag = ::dynomite::Attribute::into_attr(tag.to_owned());
                    attrs.insert(#tag_key.to_owned(), tag);
                }
            }
        }
    }

    fn unimplemented_record_variants(variant: &Variant) -> ! {
        abort!(
            variant.inner.ident.span(),
            "Record enum variants are not implemented yet."
        )
    }

    fn expect_single_item_tuple(
        tuple: &syn::FieldsUnnamed,
        variant_ident: &Ident,
    ) {
        if tuple.unnamed.len() != 1 {
            abort!(
                variant_ident,
                "Tuple variants with {} elements are not supported yet in dynomite, use \
                single-element tuples for now. \
                This restriction may be relaxed in future (follow the updates).",
                tuple.unnamed.len(),
            )
        }
    }
}

/// A Field and all its extracted dynomite derive attrs
#[derive(Clone)]
struct ItemField<'a> {
    field: &'a Field,
    attrs: Vec<FieldAttr>,
}

impl<'a> ItemField<'a> {
    fn new(field: &'a Field) -> Self {
        let attrs = parse_attrs(&field.attrs);
        let me = Self { field, attrs };
        if me.is_flatten() {
            if let Some(it) = me
                .attrs
                .iter()
                .find(|it| !matches!(it.kind, FieldAttrKind::Flatten))
            {
                abort!(
                    it.ident,
                    "If #[dynomite(flatten)] is used, no other dynomite attributes are allowed on the field"
                );
            }
        }
        me
    }

    fn is_partition_key(&self) -> bool {
        self.attrs
            .iter()
            .any(|attr| matches!(attr.kind, FieldAttrKind::PartitionKey))
    }

    fn is_sort_key(&self) -> bool {
        self.attrs
            .iter()
            .any(|attr| matches!(attr.kind, FieldAttrKind::SortKey))
    }

    fn is_default_when_absent(&self) -> bool {
        self.attrs
            .iter()
            .any(|attr| matches!(attr.kind, FieldAttrKind::Default))
    }

    fn is_flatten(&self) -> bool {
        self.attrs
            .iter()
            .any(|attr| matches!(attr.kind, FieldAttrKind::Flatten))
    }

    fn deser_name(&self) -> String {
        let ItemField { field, attrs } = self;
        attrs
            .iter()
            .find_map(|attr| match &attr.kind {
                FieldAttrKind::Rename(lit) => Some(lit.value()),
                _ => None,
            })
            .unwrap_or_else(|| {
                field
                    .ident
                    .as_ref()
                    .expect("should have an identifier")
                    .to_string()
            })
    }
}

fn parse_attrs<A: Parse>(all_attrs: &[Attribute]) -> Vec<A> {
    all_attrs
        .iter()
        .filter(|attr| is_dynomite_attr(attr))
        .flat_map(|attr| {
            attr.parse_args_with(Punctuated::<A, Token![,]>::parse_terminated)
                .unwrap_or_abort()
        })
        .collect()
}

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
#[proc_macro_error::proc_macro_error]
#[proc_macro_derive(Item, attributes(partition_key, sort_key, dynomite))]
pub fn derive_item(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input);

    let gen = match expand_item(ast) {
        Ok(g) => g,
        Err(e) => return e.to_compile_error().into(),
    };

    gen.into_token_stream().into()
}

/// similar in spirit to `#[derive(Item)]` except these are exempt from declaring
/// partition and sort keys
#[proc_macro_error::proc_macro_error]
#[proc_macro_derive(Attributes, attributes(dynomite))]
pub fn derive_attributes(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input);
    expand_attributes(ast).unwrap_or_else(|e| e.to_compile_error().into())
}

/// Derives `dynomite::Attribute` for enum types
///
/// # Panics
///
/// This proc macro will panic when applied to other types
#[proc_macro_error::proc_macro_error]
#[proc_macro_derive(Attribute)]
pub fn derive_attribute(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input);
    let gen = expand_attribute(ast);
    gen.into_token_stream().into()
}

fn expand_attribute(ast: DeriveInput) -> impl ToTokens {
    let name = &ast.ident;
    match ast.data {
        syn::Data::Enum(variants) => {
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
    variants: &[syn::Variant],
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

fn expand_attributes(ast: DeriveInput) -> syn::Result<TokenStream> {
    use syn::spanned::Spanned as _;
    let name = ast.ident;
    let tokens = match ast.data {
        syn::Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(named) => {
                make_dynomite_attrs_for_struct(&name, &named.named.into_iter().collect::<Vec<_>>())
                    .into_token_stream()
            }
            fields => {
                return Err(syn::Error::new(
                    fields.span(),
                    "Dynomite Attributes require named fields",
                ))
            }
        },
        syn::Data::Enum(data_enum) => {
            make_dynomite_attrs_for_enum(&DataEnum::new(name, data_enum, &ast.attrs))
                .into_token_stream()
        }
        _ => panic!("Dynomite Attributes can only be generated for structs"),
    };
    Ok(tokens.into())
}

fn expand_item(ast: DeriveInput) -> syn::Result<impl ToTokens> {
    use syn::spanned::Spanned as _;
    let name = &ast.ident;
    let vis = &ast.vis;
    match ast.data {
        syn::Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(named) => {
                make_dynomite_item(vis, name, &named.named.into_iter().collect::<Vec<_>>())
            }
            fields => Err(syn::Error::new(
                fields.span(),
                "Dynomite Items require named fields",
            )),
        },
        _ => panic!("Dynomite Items can only be generated for structs"),
    }
}

fn make_dynomite_attrs_for_enum(enum_item: &DataEnum) -> impl ToTokens {
    let from_attributes = enum_item.impl_from_attributes();
    let into_attributes = enum_item.impl_into_attributes();
    let std_into_attrs = get_std_convert_traits(&enum_item.ident);

    quote! {
        #from_attributes
        #into_attributes
        #std_into_attrs
    }
}

fn make_dynomite_attrs_for_struct(
    name: &Ident,
    fields: &[Field],
) -> impl ToTokens {
    let item_fields = fields.iter().map(ItemField::new).collect::<Vec<_>>();
    // impl ::dynomite::FromAttributes for Name
    let from_attribute_map = get_from_attributes_trait(name, &item_fields);
    // impl ::dynomite::IntoAttributes for Name
    // impl From<Name> for ::dynomite::Attributes
    let to_attribute_map = get_to_attribute_map_trait(name, &item_fields);
    // impl TryFrom<::dynomite::Attributes> for Name
    // impl From<Name> for ::dynomite::Attributes
    let std_into_attrs = get_std_convert_traits(name);

    quote! {
        #from_attribute_map
        #to_attribute_map
        #std_into_attrs
    }
}

fn make_dynomite_item(
    vis: &Visibility,
    name: &Ident,
    fields: &[Field],
) -> syn::Result<impl ToTokens> {
    let item_fields = fields.iter().map(ItemField::new).collect::<Vec<_>>();
    // all items must have 1 primary_key
    let partition_key_count = item_fields.iter().filter(|f| f.is_partition_key()).count();
    if partition_key_count != 1 {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "All Item's must declare one and only one partition_key. The `{}` Item declared {}",
                name, partition_key_count
            ),
        ));
    }
    // impl Item for Name + NameKey struct
    let dynamodb_traits = get_dynomite_item_traits(vis, name, &item_fields)?;
    // impl ::dynomite::FromAttributes for Name
    let from_attribute_map = get_from_attributes_trait(name, &item_fields);
    // impl ::dynomite::IntoAttributes for Name
    let to_attribute_map = get_to_attribute_map_trait(name, &item_fields);
    // impl TryFrom<::dynomite::Attributes> for Name
    // impl From<Name> for ::dynomite::Attributes
    let std_into_attrs = get_std_convert_traits(name);

    Ok(quote! {
        #from_attribute_map
        #to_attribute_map
        #std_into_attrs
        #dynamodb_traits
    })
}

fn get_to_attribute_map_trait(
    name: &Ident,
    fields: &[ItemField],
) -> impl ToTokens {
    let into_attrs = get_into_attrs(fields);

    quote! {
        impl ::dynomite::IntoAttributes for #name {
            #into_attrs
        }
    }
}

fn get_std_convert_traits(entity_name: &Ident) -> impl ToTokens {
    quote! {
        impl ::std::convert::TryFrom<::dynomite::Attributes> for #entity_name {
            type Error = ::dynomite::AttributeError;

            fn try_from(mut attrs: ::dynomite::Attributes) -> ::std::result::Result<Self, ::dynomite::AttributeError> {
                ::dynomite::FromAttributes::from_attrs(&mut attrs)
            }
        }

        impl ::std::convert::From<#entity_name> for ::dynomite::Attributes {
            fn from(entity: #entity_name) -> Self {
                let mut map = ::dynomite::Attributes::new();
                ::dynomite::IntoAttributes::into_attrs(entity, &mut map);
                map
            }
        }
    }
}

fn get_into_attrs(fields: &[ItemField]) -> impl ToTokens {
    let field_conversions = fields.iter().map(|field| {
        let field_deser_name = field.deser_name();
        let field_ident = &field.field.ident;

        if field.is_flatten() {
            quote! {
                ::dynomite::IntoAttributes::into_attrs(self.#field_ident, attrs);
            }
        } else {
            quote! {
                attrs.insert(
                    #field_deser_name.to_string(),
                    ::dynomite::Attribute::into_attr(self.#field_ident)
                );
            }
        }
    });

    quote! {
        fn into_attrs(self, attrs: &mut ::dynomite::Attributes) {
            #(#field_conversions)*
        }
    }
}

/// ```rust,ignore
/// impl ::dynomite::FromAttributes for Name {
///     fn from_attrs(attrs: &mut ::dynomite::Attributes) -> Result<Self, ::dynomite::Error> {
///         let field_name = ::dynomite::Attribute::from_attr(
///            attrs.remove("field_deser_name").ok_or_else(|| Error::MissingField { name: "field_deser_name".to_string() })?
///         );
///         Ok(Self {
///            field_name,
///         })
///     }
/// }
/// ```
fn get_from_attributes_trait(
    name: &Ident,
    fields: &[ItemField],
) -> impl ToTokens {
    let from_attrs = quote!(::dynomite::FromAttributes);
    let from_attrs_fn = get_from_attrs_function(fields);

    quote! {
        impl #from_attrs for #name {
            #from_attrs_fn
        }
    }
}

fn get_from_attrs_function(fields: &[ItemField]) -> impl ToTokens {
    let var_init_statements = fields
        .iter()
        .map(|field| {
            // field might have #[dynomite(rename = "...")] attribute
            let field_deser_name = field.deser_name();
            let field_ident = &field.field.ident;
            let expr = if field.is_default_when_absent() {
                quote! {
                    match attrs.remove(#field_deser_name) {
                        Some(field) => ::dynomite::Attribute::from_attr(field)?,
                        _ => ::std::default::Default::default()
                    }
                }
            } else if field.is_flatten() {
                quote! { ::dynomite::FromAttributes::from_attrs(attrs)? }
            } else {
                quote! {
                    ::dynomite::Attribute::from_attr(
                        attrs.remove(#field_deser_name).ok_or_else(|| ::dynomite::AttributeError::MissingField {
                            name: #field_deser_name.to_string()
                        })?
                    )?
                }
            };
            quote! {
                let #field_ident = #expr;
            }
        });

    let field_names = fields.iter().map(|it| &it.field.ident);

    // The order of evaluation of struct literal fields seems
    // **informally** left-to-right (as per Niko Matsakis and Steve Klabnik),
    // https://stackoverflow.com/a/57612600/9259330
    // This means we should not rely on this behavior yet.
    // We explicitly make conversion expressions a separate statements.
    // This is important, because the order of declaration and evaluation
    // of `flatten` fields matters.

    quote! {
        fn from_attrs(attrs: &mut ::dynomite::Attributes) -> ::std::result::Result<Self, ::dynomite::AttributeError> {
            #(#var_init_statements)*
            ::std::result::Result::Ok(Self {
                #(#field_names),*
            })
        }
    }
}

fn get_dynomite_item_traits(
    vis: &Visibility,
    name: &Ident,
    fields: &[ItemField],
) -> syn::Result<impl ToTokens> {
    let impls = get_item_impls(vis, name, fields)?;

    Ok(quote! {
        #impls
    })
}

fn get_item_impls(
    vis: &Visibility,
    name: &Ident,
    fields: &[ItemField],
) -> syn::Result<impl ToTokens> {
    // impl ::dynomite::Item for Name ...
    let item_trait = get_item_trait(name, fields)?;
    // pub struct NameKey ...
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
    fields: &[ItemField],
) -> syn::Result<impl ToTokens> {
    let item = quote!(::dynomite::Item);
    let attribute_map = quote!(
        ::std::collections::HashMap<String, ::dynomite::dynamodb::AttributeValue>
    );
    let partition_key_field = fields.iter().find(|f| f.is_partition_key());
    let sort_key_field = fields.iter().find(|f| f.is_sort_key());
    let partition_key_insert = partition_key_field.map(get_key_inserter).transpose()?;
    let sort_key_insert = sort_key_field.map(get_key_inserter).transpose()?;

    let partition_key_tuple = partition_key_field.map(get_key_tuple);
    let sort_key_tuple = sort_key_field
        .map(get_key_tuple)
        .map(|tuple| quote! { Some(#tuple) })
        .unwrap_or_else(|| quote! { None });

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

                    fn partition_key(&self) -> (String, ::dynomite::dynamodb::AttributeValue) {
                        #partition_key_tuple
                    }

                    fn sort_key(&self) -> Option<(String, ::dynomite::dynamodb::AttributeValue)> {
                        #sort_key_tuple
                    }
                }
            }
        })
        .unwrap_or_else(proc_macro2::TokenStream::new))
}

/// ```rust,ignore
/// keys.insert(
///   "field_deser_name", to_attribute_value(field)
/// );
/// ```
fn get_key_inserter(field: &ItemField) -> syn::Result<impl ToTokens> {
    let to_attribute_value = quote!(::dynomite::Attribute::into_attr);

    let field_deser_name = field.deser_name();
    let field_ident = &field.field.ident;
    Ok(quote! {
        keys.insert(
            #field_deser_name.to_string(),
            #to_attribute_value(self.#field_ident.clone())
        );
    })
}

/// ```rust,ignore
/// ("field_deser_name", to_attribute_value(field))
/// ```
fn get_key_tuple(field: &ItemField) -> impl ToTokens {
    let to_attribute_value = quote!(::dynomite::Attribute::into_attr);

    let field_deser_name = field.deser_name();
    let field_ident = &field.field.ident;
    quote! {
        (#field_deser_name.to_string(), #to_attribute_value(self.#field_ident.clone()))
    }
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
    fields: &[ItemField],
) -> syn::Result<impl ToTokens> {
    let name = Ident::new(&format!("{}Key", name), Span::call_site());

    let partition_key_field = fields
        .iter()
        .find(|field| field.is_partition_key())
        .cloned()
        .map(|field| {
            // clone because this is a new struct
            // note: this in inherits field attrs so that
            // we retain dynomite(rename = "xxx")
            let mut field = field.field.clone();
            field.attrs.retain(is_dynomite_attr);

            quote! {
                #field
            }
        });

    let sort_key_field = fields
        .iter()
        .find(|field| field.is_sort_key())
        .cloned()
        .map(|field| {
            // clone because this is a new struct
            // note: this in inherits field attrs so that
            // we retain dynomite(rename = "xxx")
            let mut field = field.field.clone();
            field.attrs.retain(is_dynomite_attr);

            quote! {
                #field
            }
        });

    Ok(partition_key_field
        .map(|partition_key_field| {
            quote! {
                #[derive(::dynomite::Attributes, Debug, Clone, PartialEq)]
                #vis struct #name {
                    #partition_key_field,
                    #sort_key_field
                }
            }
        })
        .unwrap_or_else(proc_macro2::TokenStream::new))
}

fn is_dynomite_attr(suspect: &syn::Attribute) -> bool {
    suspect.path.is_ident("dynomite")
}
