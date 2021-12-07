use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::*;

use crate::common::{parse_boolean, parse_ident_as_string, parse_string};

pub struct Command {
    pub name: String,
    pub description: String,
    pub options: Vec<CommandOption>,
    pub is_subcommand_container: bool,
}

#[derive(Clone, Copy)]
pub enum OptionKind {
    Boolean,
    String,
    Integer,
    Number,
    Mention,
    User,
    Channel,
    Role,
    SubCommand,
    SubCommandGroup,
}

impl OptionKind {
    fn new_command(s: &str) -> Option<Self> {
        Some(match s {
            "boolean" => Self::Boolean,
            "string" => Self::String,
            "integer" => Self::Integer,
            "number" => Self::Number,
            "mention" => Self::Mention,
            "user" => Self::User,
            "channel" => Self::Channel,
            "role" => Self::Role,
            _ => return None,
        })
    }

    fn new_subcommand(s: &str) -> Option<Self> {
        Some(match s {
            "subcommand" => Self::SubCommand,
            "group" => Self::SubCommandGroup,
            _ => return None,
        })
    }

    pub fn to_subcommand_registration_fn(self) -> TokenStream {
        match self {
            Self::SubCommand => quote!(register_subcommand),
            Self::SubCommandGroup => quote!(register_subcommand_group),
            _ => unreachable!(),
        }
    }

    pub fn to_subcommand_parsing_fn(self) -> TokenStream {
        match self {
            Self::SubCommand => quote!(parse_subcommand),
            Self::SubCommandGroup => quote!(parse_subcommand_group),
            _ => unreachable!(),
        }
    }

    pub fn to_data_option_value_extraction(self) -> TokenStream {
        match self {
            Self::User => quote!(User(v, _)),
            Self::SubCommand | Self::SubCommandGroup => unreachable!(),
            _ => quote!(#self(v)),
        }
    }
}

impl ToTokens for OptionKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            OptionKind::Boolean => quote!(Boolean),
            OptionKind::String => quote!(String),
            OptionKind::Integer => quote!(Integer),
            OptionKind::Number => quote!(Number),
            OptionKind::Mention => quote!(Mention),
            OptionKind::User => quote!(User),
            OptionKind::Channel => quote!(Channel),
            OptionKind::Role => quote!(Role),
            OptionKind::SubCommand => quote!(SubCommand),
            OptionKind::SubCommandGroup => quote!(SubCommandGroup),
        });
    }
}

pub struct CommandOption {
    pub ident: Ident,
    pub name: Option<String>,
    pub description: Option<String>,
    pub kind: OptionKind,
    pub required: bool,
}

pub fn parse_command(input: &DeriveInput) -> Result<Command> {
    let mut name = None;
    let mut description = None;

    for attr in &input.attrs {
        if attr.path.is_ident("doc") {
            if description.is_some() {
                return Err(Error::new(attr.span(), "documentation string has already been provided"));
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

        if attr.path.is_ident("command") {
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

    let (options, is_subcommand_container) = match &input.data {
        Data::Struct(s) => (parse_struct(s)?, false),
        Data::Enum(e) => (parse_enum(e)?, true),
        _ => {
            return Err(Error::new(
                input.span(),
                "expected either a struct with named fields or an enum",
            ))
        }
    };

    let name = name.ok_or(Error::new(
        input.ident.span(),
        "expected `#[command(...)]` attribute",
    ))?;

    let description = description.ok_or(Error::new(
        input.ident.span(),
        "expected a description in documentation string",
    ))?;

    Ok(Command {
        name,
        description,
        options,
        is_subcommand_container,
    })
}

fn parse_struct(data: &DataStruct) -> Result<Vec<CommandOption>> {
    let mut options = Vec::new();

    match &data.fields {
        Fields::Unit => return Ok(options),
        Fields::Named(n) => {
            for field in &n.named {
                options.push(parse_command_option(
                    field.span(),
                    field.ident.clone().unwrap(),
                    &field.attrs,
                )?);
            }
        }
        _ => {
            return Err(Error::new(
                data.fields.span(),
                "expected a struct with named fields or a unit struct",
            ))
        }
    };

    Ok(options)
}

fn parse_enum(data: &DataEnum) -> Result<Vec<CommandOption>> {
    let mut options = Vec::new();

    for variant in &data.variants {
        options.push(parse_subcommand_option(
            variant.span(),
            variant.ident.clone(),
            &variant.attrs,
        )?);
    }

    Ok(options)
}

fn parse_command_option(span: Span, ident: Ident, attrs: &[Attribute]) -> Result<CommandOption> {
    let mut name = None;
    let mut description = None;
    let mut kind = None;
    let mut required = None;

    for attr in attrs {
        let list = match attr.parse_meta()? {
            Meta::List(l) => l,
            Meta::NameValue(nv) => {
                if nv.path.is_ident("doc") {
                    if description.is_some() {
                        return Err(Error::new(nv.span(), "documentation string has already been provided"));
                    }

                    description = Some(match nv.lit {
                        Lit::Str(s) => s.value().trim().to_string(),
                        _ => return Err(Error::new(nv.span(), "expected string")),
                    });

                    continue;
                }

                return Err(Error::new(nv.span(), "expected documentation string"));
            }
            Meta::Path(..) => return Err(Error::new(attr.span(), "expected a list")),
        };

        for meta in list.nested {
            match meta {
                NestedMeta::Meta(Meta::List(_)) | NestedMeta::Lit(_) => {
                    return Err(Error::new(
                        meta.span(),
                        "expected `<name>` or `<name> = <value>` parameters",
                    ))
                }
                NestedMeta::Meta(m) => {
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

                    if let Some(r) = parse_boolean(&m, "required")? {
                        if required.is_some() {
                            return Err(Error::new(
                                m.span(),
                                "`required` parameter has already been provided",
                            ));
                        }

                        required = Some(r);
                        continue;
                    }

                    if kind.is_some() {
                        return Err(Error::new(
                            m.span(),
                            "option type parameter has already been provided",
                        ));
                    }

                    if let Some(s) = parse_ident_as_string(&m) {
                        kind = OptionKind::new_command(&s);
                    }
                }
            };
        }
    }

    if description.is_none() {
        return Err(Error::new(span, "expected documentation string"));
    }

    let kind = kind.ok_or(Error::new(
        span,
        "expected a valid option type (e.g. `string`, `integer`)",
    ))?;

    let required = required.unwrap_or(false);

    Ok(CommandOption {
        ident,
        name,
        description,
        kind,
        required,
    })
}

fn parse_subcommand_option(span: Span, ident: Ident, attrs: &[Attribute]) -> Result<CommandOption> {
    let mut kind = None;
    let mut required = None;

    for attr in attrs {
        let list = match attr.parse_meta()? {
            Meta::List(l) => l,
            _ => return Err(Error::new(attr.span(), "expected a list")),
        };

        for meta in list.nested {
            match meta {
                NestedMeta::Meta(Meta::List(_)) | NestedMeta::Lit(_) => {
                    return Err(Error::new(meta.span(), "expected `<name>` parameters"))
                }
                NestedMeta::Meta(m) => {
                    if let Some(r) = parse_boolean(&m, "required")? {
                        if required.is_some() {
                            return Err(Error::new(
                                m.span(),
                                "`required` parameter has already been provided",
                            ));
                        }

                        required = Some(r);
                        continue;
                    }

                    if kind.is_some() {
                        return Err(Error::new(
                            m.span(),
                            "option type parameter has already been provided",
                        ));
                    }

                    if let Some(s) = parse_ident_as_string(&m) {
                        kind = OptionKind::new_subcommand(&s);
                    }
                }
            };
        }
    }

    let kind = kind.ok_or(Error::new(
        span,
        "expected a valid option type (`subcommand`, `group`)",
    ))?;

    let required = required.unwrap_or(false);

    Ok(CommandOption {
        ident,
        name: None,
        description: None,
        kind,
        required,
    })
}
