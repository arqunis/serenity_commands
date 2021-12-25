use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::*;

use crate::common::{get_lit_string, get_path_as_string, is_option, parse_doc, AttrOption};

pub struct Command {
    pub name: String,
    pub description: String,
    pub data: CommandData,
}

pub enum CommandData {
    Options(Vec<CommandOption>),
    SubCommands(Vec<SubCommand>),
}

impl Command {
    pub fn new(input: &DeriveInput) -> Result<Command> {
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

            if !attr.path.is_ident("command") {
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
                    "expected a description in documentation string",
                ));
            },
        };

        let data = match &input.data {
            Data::Struct(s) => parse_struct(s)?,
            Data::Enum(e) => parse_enum(e)?,
            _ => {
                return Err(Error::new(
                    input.span(),
                    "expected either a struct with named fields or an enum",
                ))
            },
        };

        Ok(Command {
            name,
            description,
            data,
        })
    }
}

fn parse_struct(data: &DataStruct) -> Result<CommandData> {
    match &data.fields {
        Fields::Unit => Ok(CommandData::Options(Vec::new())),
        Fields::Named(n) => {
            if n.named.len() > 25 {
                return Err(Error::new(
                    data.fields.span(),
                    "a command cannot have more than 25 options",
                ));
            }

            let mut options = Vec::new();
            for field in &n.named {
                options.push(CommandOption::new(field)?);
            }

            Ok(CommandData::Options(options))
        },
        _ => {
            return Err(Error::new(
                data.fields.span(),
                "expected a struct with named fields or a unit struct",
            ));
        },
    }
}

fn parse_enum(data: &DataEnum) -> Result<CommandData> {
    if data.variants.len() > 25 {
        return Err(Error::new(
            data.variants.span(),
            "a command cannot have more than 25 of the combined sum of subcommand groups and subcommands",
        ));
    }

    let mut subcommands = Vec::new();

    for variant in &data.variants {
        ensure_tuple_variant(variant)?;

        subcommands.push(SubCommand::new(variant)?);
    }

    Ok(CommandData::SubCommands(subcommands))
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

#[derive(Clone, Copy)]
pub enum CommandOptionKind {
    Boolean,
    String,
    Integer,
    Number,
    Mention,
    User,
    Channel,
    Role,
}

impl CommandOptionKind {
    fn new(s: &str) -> Option<Self> {
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

    pub fn to_data_option_value_extraction(self) -> TokenStream {
        match self {
            Self::User => quote!(User(v, _)),
            _ => quote!(#self(v)),
        }
    }
}

impl ToTokens for CommandOptionKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::Boolean => quote!(Boolean),
            Self::String => quote!(String),
            Self::Integer => quote!(Integer),
            Self::Number => quote!(Number),
            Self::Mention => quote!(Mention),
            Self::User => quote!(User),
            Self::Channel => quote!(Channel),
            Self::Role => quote!(Role),
        });
    }
}
pub struct CommandOption {
    pub ident: Ident,
    pub ty: Type,
    pub required: bool,
    pub name: String,
    pub description: String,
    pub kind: CommandOptionKind,
}

impl CommandOption {
    fn new(field: &Field) -> Result<Self> {
        let ident = match &field.ident {
            Some(i) => i.clone(),
            None => {
                return Err(Error::new(field.span(), "expected a name field (i.e. `name: type`)"))
            },
        };

        let mut name = AttrOption::new("name");

        let mut description = None;
        let mut kind = None;

        for attr in &field.attrs {
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

            if !attr.path.is_ident("option") {
                continue;
            }

            let list = match attr.parse_meta()? {
                Meta::List(l) => l,
                _ => return Err(Error::new(attr.span(), "expected a list")),
            };

            for meta in list.nested {
                match &meta {
                    NestedMeta::Lit(_) => {
                        return Err(Error::new(meta.span(), "unexpected literal"));
                    },
                    NestedMeta::Meta(m) => match m {
                        // `name = "..."` option
                        Meta::NameValue(nv) if nv.path.is_ident("name") => {
                            name.set(nv.span(), get_lit_string(&nv.lit)?)?;
                        },

                        // `boolean` | `string` | `integer` | `number` | `mention` | `user` | `channel` | `role` option
                        Meta::Path(p) => {
                            if kind.is_some() {
                                return Err(Error::new(
                                    p.span(),
                                    "option type has already been provided",
                                ));
                            }

                            kind = CommandOptionKind::new(&get_path_as_string(p)?);
                        },
                        _ => {
                            return Err(Error::new(meta.span(), "unknown option or invalid syntax"))
                        },
                    },
                };
            }
        }

        let name = name.value().unwrap_or_else(|| ident.to_string());

        let description = match description {
            Some(desc) => desc,
            None => {
                return Err(Error::new(
                    field.span(),
                    "expected a documentation string for the description",
                ))
            },
        };

        let kind = match kind {
            Some(kind) => kind,
            None => {
                return Err(Error::new(
                    field.span(),
                    "expected a type for the option (e.g. `string`, `integer`, `number`, ...)",
                ))
            },
        };

        Ok(Self {
            ident,
            ty: field.ty.clone(),
            required: !is_option(&field.ty),
            name,
            description,
            kind,
        })
    }
}

#[derive(Clone, Copy)]
pub enum SubCommandKind {
    SubCommand,
    Group,
}

impl SubCommandKind {
    fn new(s: &str) -> Option<Self> {
        Some(match s {
            "subcommand" => Self::SubCommand,
            "group" => Self::Group,
            _ => return None,
        })
    }

    pub fn to_registration_fn(self) -> TokenStream {
        match self {
            Self::SubCommand => quote!(register_subcommand),
            Self::Group => quote!(register_subcommand_group),
        }
    }

    pub fn to_parsing_fn(self) -> TokenStream {
        match self {
            Self::SubCommand => quote!(parse_subcommand),
            Self::Group => quote!(parse_subcommand_group),
        }
    }
}

impl ToTokens for SubCommandKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::SubCommand => quote!(SubCommand),
            Self::Group => quote!(SubCommandGroup),
        });
    }
}

pub struct SubCommand {
    pub ident: Ident,
    pub kind: SubCommandKind,
}

impl SubCommand {
    fn new(var: &Variant) -> Result<Self> {
        let ident = var.ident.clone();

        let mut kind = None;

        for attr in &var.attrs {
            if !attr.path.is_ident("option") {
                continue;
            }

            let list = match attr.parse_meta()? {
                Meta::List(l) => l,
                _ => return Err(Error::new(attr.span(), "expected a list")),
            };

            for meta in list.nested {
                match &meta {
                    NestedMeta::Lit(_) => {
                        return Err(Error::new(meta.span(), "unexpected literal"));
                    },
                    NestedMeta::Meta(m) => match m {
                        // `subcommand` | `group` option
                        Meta::Path(p) => {
                            if kind.is_some() {
                                return Err(Error::new(
                                    p.span(),
                                    "option type has already been provided",
                                ));
                            }

                            kind = SubCommandKind::new(&get_path_as_string(p)?);
                        },
                        _ => {
                            return Err(Error::new(meta.span(), "unknown option or invalid syntax"))
                        },
                    },
                };
            }
        }

        let kind = match kind {
            Some(kind) => kind,
            None => {
                return Err(Error::new(
                    var.span(),
                    "expected a type for the option (either `subcommand` or `group`)",
                ));
            },
        };

        Ok(Self {
            ident,
            kind,
        })
    }
}
