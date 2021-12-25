use proc_macro2::Ident;
use syn::spanned::Spanned;
use syn::*;

use crate::common::ensure_tuple_variant;

pub struct Commands {
    pub commands: Vec<Ident>,
}

pub fn parse_commands(input: &DeriveInput) -> Result<Commands> {
    let commands = match &input.data {
        Data::Enum(e) => parse_enum(e)?,
        _ => return Err(Error::new(input.span(), "expected an enum")),
    };

    Ok(Commands {
        commands,
    })
}

fn parse_enum(data: &DataEnum) -> Result<Vec<Ident>> {
    let mut idents = Vec::new();

    for variant in &data.variants {
        ensure_tuple_variant(variant, "command")?;

        idents.push(variant.ident.clone());
    }

    Ok(idents)
}
