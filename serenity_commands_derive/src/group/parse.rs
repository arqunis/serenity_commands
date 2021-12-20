use proc_macro2::Ident;
use syn::spanned::Spanned;
use syn::*;

use crate::common::{get_lit_string, parse_doc, AttrOption};

pub struct Group {
    pub name: String,
    pub description: String,
    pub subcommands: Vec<Ident>,
}

pub fn parse_group(input: &DeriveInput) -> Result<Group> {
    let mut name = AttrOption::new("name");

    let mut description = None;

    for attr in &input.attrs {
        if attr.path.is_ident("doc") {
            if description.is_some() {
                return Err(Error::new(
                    attr.span(),
                    "documentation string has already been provided",
                ));
            }

            description = Some(parse_doc(attr)?);
            continue;
        }

        if !attr.path.is_ident("group") {
            continue;
        }

        let list = match attr.parse_meta()? {
            Meta::List(l) => l,
            _ => return Err(Error::new(attr.span(), "expected a list")),
        };

        for meta in list.nested {
            match meta {
                NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("name") => {
                    name.set(nv.span(), get_lit_string(&nv.lit)?)?;
                },
                _ => return Err(Error::new(meta.span(), "unknown option or invalid syntax")),
            };
        }
    }

    let subcommands = match &input.data {
        Data::Enum(e) => parse_enum(e),
        _ => return Err(Error::new(input.span(), "expected an enum")),
    };

    let name = match name.value() {
        Some(name) => name,
        None => {
            return Err(Error::new(input.ident.span(), "expected a name"));
        },
    };

    let description = match description {
        Some(desc) => desc,
        None => {
            return Err(Error::new(
                input.ident.span(),
                "expected a description in a documentation string",
            ));
        },
    };

    Ok(Group {
        name,
        description,
        subcommands,
    })
}

fn parse_enum(data: &DataEnum) -> Vec<Ident> {
    data.variants.iter().map(|v| v.ident.clone()).collect()
}
