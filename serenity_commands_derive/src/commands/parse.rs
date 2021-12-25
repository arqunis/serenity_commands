use proc_macro2::Ident;
use syn::spanned::Spanned;
use syn::*;

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
        ensure_tuple_variant(variant)?;

        idents.push(variant.ident.clone());
    }

    Ok(idents)
}

fn ensure_tuple_variant(variant: &Variant) -> Result<()> {
    match &variant.fields {
        Fields::Unnamed(n) if n.unnamed.len() != 1 => Err(Error::new(
            n.span(),
            "expected a single command as a field of this tuple struct variant",
        )),
        Fields::Unnamed(_) => Ok(()),
        _ => {
            let mut err = Error::new(
                variant.span(),
                "expected a command as a field in a tuple struct variant",
            );

            err.combine(Error::new(
                variant.span(),
                format_args!("note: try changing this to `{0}({0})`", variant.ident),
            ));

            Err(err)
        },
    }
}
