use proc_macro2::Ident;
use syn::spanned::Spanned;
use syn::*;

pub struct Commands {
    pub commands: Vec<Ident>,
}

pub fn parse_commands(input: &DeriveInput) -> Result<Commands> {
    let commands = match &input.data {
        Data::Enum(e) => parse_enum(e),
        _ => return Err(Error::new(input.span(), "expected an enum")),
    };

    Ok(Commands {
        commands,
    })
}

fn parse_enum(data: &DataEnum) -> Vec<Ident> {
    data.variants.iter().map(|v| v.ident.clone()).collect()
}
