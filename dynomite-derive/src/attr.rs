//! dynomite field attributes

use proc_macro_error::abort;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Token,
};

#[derive(Clone)]
pub enum Attr {
    /// Denotes field should be replaced with Default impl when absent in ddb
    Default(Ident),
    /// Denotes field should be renamed to value of ListStr
    Rename(Ident, LitStr),
    /// Denotes Item partition (primary) key
    PartitionKey(Ident),
    /// Denotes Item sort key
    SortKey(Ident),
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use self::Attr::*;
        let name: Ident = input.parse()?;
        let name_str = name.to_string();
        if input.peek(Token![=]) {
            // `name = value` attributes.
            let assign = input.parse::<Token![=]>()?; // skip '='
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                match &*name_str {
                    "rename" => Ok(Rename(name, lit)),
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
            match name_str.as_ref() {
                "default" => Ok(Default(name)),
                "partition_key" => Ok(PartitionKey(name)),
                "sort_key" => Ok(SortKey(name)),
                _ => abort!(name, "unexpected dynomite attribute: {}", name_str),
            }
        }
    }
}
