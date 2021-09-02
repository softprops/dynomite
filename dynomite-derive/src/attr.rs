//! dynomite field attributes

use proc_macro_error::abort;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Path, Token,
};

/// Represents a parsed attribute that appears in `#[dynomite(...)]`.
#[derive(Clone)]
pub(crate) struct Attr<Kind> {
    /// The identifier part of the attribute (e.g. `rename` in `#[dynomite(rename = "foo"`))
    pub(crate) ident: Ident,
    /// More specific information about the metadata entry.
    pub(crate) kind: Kind,
}

/// Attribute that appears on record fields (struct fields and enum record variant fields)
pub(crate) type FieldAttr = Attr<FieldAttrKind>;
/// Attribute that appears on the top level of an enum
pub(crate) type EnumAttr = Attr<EnumAttrKind>;
/// Attribute that appears on enum varinats
pub(crate) type VariantAttr = Attr<VariantAttrKind>;

#[derive(Clone)]
pub(crate) enum FieldAttrKind {
    /// Denotes field should be replaced with Default impl when absent in ddb
    Default,

    /// Denotes field should be renamed to value of ListStr
    Rename(LitStr),

    /// Denotes Item partition (primary) key
    PartitionKey,

    /// Denotes Item sort key
    SortKey,

    /// Denotes a field that should be replaced with all of its subfields
    Flatten,

    /// Denotes a field that should not be present in the resulting `Attributes` map
    /// if the given function returns `true` for its value
    SkipSerializingIf(Path),
}

impl DynomiteAttr for FieldAttrKind {
    const KVS: Kvs<Self> = &[
        ("rename", |lit| Ok(FieldAttrKind::Rename(lit))),
        ("skip_serializing_if", |lit| {
            lit.parse().map(FieldAttrKind::SkipSerializingIf)
        }),
    ];
    const KEYS: Keys<Self> = &[
        ("default", FieldAttrKind::Default),
        ("partition_key", FieldAttrKind::PartitionKey),
        ("sort_key", FieldAttrKind::SortKey),
        ("flatten", FieldAttrKind::Flatten),
    ];
}

#[derive(Clone)]
pub(crate) enum EnumAttrKind {
    // FIXME: implement content attribute to support non-map values in enum variants
    // (adjacently tagged enums: https://serde.rs/enum-representations.html#adjacently-tagged)
    // Content(LitStr),
    /// The name of the tag field for an internally-tagged enum
    Tag(LitStr),
}

impl DynomiteAttr for EnumAttrKind {
    const KVS: Kvs<Self> = &[("tag", |lit| Ok(EnumAttrKind::Tag(lit)))];
}

#[derive(Clone)]
pub(crate) enum VariantAttrKind {
    // TODO: add default for enum variants?
    Rename(LitStr),
}

impl DynomiteAttr for VariantAttrKind {
    const KVS: Kvs<Self> = &[("rename", |lit| Ok(VariantAttrKind::Rename(lit)))];
}

type Kvs<T> = &'static [(&'static str, fn(syn::LitStr) -> syn::Result<T>)];
type Keys<T> = &'static [(&'static str, T)];

/// Helper to ease defining `#[dynomite(key)` and `#[dynomite(key = "val")` attributes
pub(crate) trait DynomiteAttr: Clone + Sized + 'static {
    /// List of `("attr_name", enum_variant_constructor)` to define attributes
    /// that require a value string literal (e.g. `rename = "foo"`)
    const KVS: Kvs<Self> = &[];
    /// List of `("attr_name", enum_variant_value)` entires to define attributes
    /// that should not have any value (e.g. `default` or `flatten`)
    const KEYS: Keys<Self> = &[];
}

impl<A: DynomiteAttr> Parse for Attr<A> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let entry: MetadataEntry = input.parse()?;
        let kind = entry
            .try_attr_with_val(A::KVS)?
            .or_else(|| entry.try_attr_without_val(A::KEYS))
            .unwrap_or_else(|| abort!(entry.key, "unexpected dynomite attribute: {}", entry.key));
        Ok(Attr {
            ident: entry.key,
            kind,
        })
    }
}

struct MetadataEntry {
    key: Ident,
    val: Option<LitStr>,
}

impl MetadataEntry {
    /// Attempt to map the parsed entry to an identifier-only attribute from the list
    fn try_attr_without_val<T: Clone>(
        &self,
        mappings: Keys<T>,
    ) -> Option<T> {
        let Self { key, val } = self;
        let key_str = key.to_string();
        mappings
            .iter()
            .find(|(key_pat, _)| *key_pat == key_str)
            .map(|(_, enum_val)| match val {
                Some(_) => abort!(key, "expected no value for dynomite attribute `{}`", key),
                None => enum_val.clone(),
            })
    }

    /// Attempt to map the parsed entry to a key-value attribute from the list
    fn try_attr_with_val<T>(
        &self,
        mappings: Kvs<T>,
    ) -> syn::Result<Option<T>> {
        let Self { key, val } = self;
        let key_str = key.to_string();
        mappings
            .iter()
            .find(|(key_pat, _)| *key_pat == key_str)
            .map(|(_, to_enum)| match val {
                Some(it) => to_enum(it.clone()),
                None => abort!(
                    key,
                    "expected a value for dynomite attribute: `{} = \"foo\"`",
                    key
                ),
            })
            .transpose()
    }
}

impl Parse for MetadataEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if input.peek(syn::token::Paren) {
            // `name(...)` attributes.
            abort!(key, "unexpected paren in dynomite attribute: {}", key);
        }
        Ok(Self {
            key,
            val: input
                .parse::<Token![=]>()
                .ok()
                .map(|_| input.parse())
                .transpose()?,
        })
    }
}
