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
        Data::Enum(e) => parse_enum(e)?,
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

fn parse_enum(data: &DataEnum) -> Result<Vec<Ident>> {
    if data.variants.len() > 25 {
        return Err(Error::new(
            data.variants.span(),
            "a subcommand group cannot have more than 25 subcommands",
        ));
    }

    let mut idents = Vec::new();

    for variant in &data.variants {
        ensure_tuple_variant(variant)?;

        idents.push(variant.ident.clone());
    }

    Ok(idents)
}

fn ensure_tuple_variant(variant: &Variant) -> Result<()> {
    match &variant.fields {
        Fields::Unnamed(n) if n.unnamed.len() != 1 => Err(Error::new(
            n.span(),
            "expected a single subcommand as a field of this tuple struct variant",
        )),
        Fields::Unnamed(_) => Ok(()),
        _ => {
            let mut err = Error::new(
                variant.span(),
                "expected a subcommand as a field in a tuple struct variant",
            );

            err.combine(Error::new(
                variant.span(),
                format_args!("note: try changing this to `{0}({0})`", variant.ident),
            ));

            Err(err)
        },
    }
}
