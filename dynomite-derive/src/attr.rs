//! dynomite field attributes

use proc_macro_error::abort;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Token,
};

#[derive(Clone)]
pub(crate) struct Attr {
    pub(crate) ident: Ident,
    pub(crate) kind: AttrKind,
}

#[derive(Clone)]
pub(crate) enum AttrKind {
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
}

impl Attr {
    fn new(
        ident: Ident,
        kind: AttrKind,
    ) -> Self {
        Self { ident, kind }
    }
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();
        if input.peek(Token![=]) {
            // `name = value` attributes.
            let assign = input.parse::<Token![=]>()?; // skip '='
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                match &*name_str {
                    "rename" => Ok(Attr::new(name, AttrKind::Rename(lit))),
                    unsupported => abort! {
                        name,
                        "unsupported dynomite {} attribute",
                        unsupported
                    },
                }
            } else {
                abort! {
                    assign,
                    "expected `string literal` after `=`"
                };
            }
        } else if input.peek(syn::token::Paren) {
            // `name(...)` attributes.
            abort!(name, "unexpected dynomite attribute: {}", name_str);
        } else {
            // Attributes represented with a sole identifier.
            let kind = match name_str.as_ref() {
                "default" => AttrKind::Default,
                "partition_key" => AttrKind::PartitionKey,
                "sort_key" => AttrKind::SortKey,
                "flatten" => AttrKind::Flatten,
                _ => abort!(name, "unexpected dynomite attribute: {}", name_str),
            };
            Ok(Attr::new(name, kind))
        }
    }
}
