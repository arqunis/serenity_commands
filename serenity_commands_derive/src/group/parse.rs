use proc_macro2::Ident;
use syn::spanned::Spanned;
use syn::*;

use crate::common::parse_string;

pub struct Group {
    pub name: String,
    pub description: String,
    pub subcommands: Vec<Ident>,
}

pub fn parse_group(input: &DeriveInput) -> Result<Group> {
    let mut name = None;
    let mut description = None;

    for attr in &input.attrs {
        if attr.path.is_ident("doc") {
            if description.is_some() {
                return Err(Error::new(
                    attr.span(),
                    "documentation string has already been provided",
                ));
            }

            let nv = match attr.parse_meta()? {
                Meta::NameValue(nv) => nv,
                _ => return Err(Error::new(attr.span(), "invalid documentation string")),
            };

            description = Some(match nv.lit {
                Lit::Str(s) => s.value().trim().to_string(),
                _ => return Err(Error::new(nv.span(), "expected string")),
            });

            continue;
        }

        if attr.path.is_ident("group") {
            let list = match attr.parse_meta()? {
                Meta::List(l) => l,
                _ => return Err(Error::new(attr.span(), "expected a list")),
            };

            for meta in list.nested {
                let m = match meta {
                    NestedMeta::Meta(m @ Meta::NameValue(_)) => m,
                    _ => {
                        return Err(Error::new(
                            meta.span(),
                            "expected `<name> = <value>` parameters",
                        ))
                    }
                };

                if let Some(s) = parse_string(&m, "name")? {
                    if name.is_some() {
                        return Err(Error::new(
                            m.span(),
                            "`name` parameter has already been provided",
                        ));
                    }

                    name = Some(s);
                    continue;
                }
            }

            continue;
        }
    }

    let subcommands = match &input.data {
        Data::Enum(e) => parse_enum(e),
        _ => return Err(Error::new(input.span(), "expected an enum")),
    };

    let name = name.ok_or(Error::new(
        input.ident.span(),
        "expected `#[command(...)]` attribute",
    ))?;

    let description = description.ok_or(Error::new(
        input.ident.span(),
        "expected a description in a documentation string",
    ))?;

    Ok(Group {
        name,
        description,
        subcommands,
    })
}

fn parse_enum(data: &DataEnum) -> Vec<Ident> {
    data.variants.iter().map(|v| v.ident.clone()).collect()
}
