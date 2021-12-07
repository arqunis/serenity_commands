use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::*;

mod parse;

use parse::*;

pub fn derive_command(item: TokenStream) -> Result<TokenStream> {
    let input = parse2::<DeriveInput>(item)?;

    let Command {
        name: cmd,
        description,
        options,
        is_subcommand_container,
    } = parse_command(&input)?;

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let extra = {
        let mut extra = TokenStream::new();

        if is_subcommand_container {
            let option_idents = options.iter().map(|o| &o.ident).collect::<Vec<_>>();

            let option_vars = options
                .iter()
                .map(|o| format_ident!("{}", o.ident.to_string().to_lowercase()))
                .collect::<Vec<_>>();

            let option_registration_fns = options
                .iter()
                .map(|o| o.kind.to_subcommand_registration_fn());

            let option_parsing_fns = options.iter().map(|o| o.kind.to_subcommand_parsing_fn());

            extra.extend(quote! {
                fn register_command(
                    cmd: &mut serenity_commands::serenity::builder::CreateApplicationCommand
                ) -> &mut serenity_commands::serenity::builder::CreateApplicationCommand {
                    cmd.name(Self::name())
                        .description(Self::description())
                        #(.create_option(#option_idents::#option_registration_fns))*
                }

                fn parse_command(
                    data: serenity_commands::serenity::model::interactions::application_command::ApplicationCommandInteractionData
                ) -> std::result::Result<Self, serenity_commands::error::ParseError> {
                    if data.name != Self::name() {
                        return Err(serenity_commands::error::ParseError::UnknownCommand(data.name.clone()));
                    }

                    #(let #option_vars = #option_idents::name();)*

                    for opt in data.options {
                        #(if opt.name == #option_vars {
                            return Ok(Self::#option_idents(#option_idents::#option_parsing_fns(opt)?));
                        })*

                        return Err(serenity_commands::error::ParseError::UnknownSubCommand(opt.name.clone()));
                    }

                    unreachable!()
                }
            });
        } else {
            let mut option_fn_names = Vec::new();

            for opt in &options {
                option_fn_names.push(generate_option_registration_fn(opt, &mut extra));
            }

            let option_idents = options.iter().map(|o| &o.ident).collect::<Vec<_>>();

            let option_names = options
                .iter()
                .map(|o| o.name.clone().unwrap_or_else(|| o.ident.to_string()))
                .collect::<Vec<_>>();

            let option_kinds = options.iter().map(|o| o.kind);

            let option_data_extractions = options
                .iter()
                .map(|o| o.kind.to_data_option_value_extraction());

            extra.extend(quote! {
                fn register_command(
                    cmd: &mut serenity_commands::serenity::builder::CreateApplicationCommand
                ) -> &mut serenity_commands::serenity::builder::CreateApplicationCommand {
                    cmd.name(Self::name())
                        .description(Self::description())
                        #(.create_option(Self::#option_fn_names))*
                }

                fn register_subcommand(
                    opt: &mut serenity_commands::serenity::builder::CreateApplicationCommandOption
                ) -> &mut serenity_commands::serenity::builder::CreateApplicationCommandOption {
                    use serenity_commands::serenity::model::interactions::application_command::ApplicationCommandOptionType;

                    opt.name(Self::name())
                        .description(Self::description())
                        .kind(ApplicationCommandOptionType::SubCommand)
                        #(.create_sub_option(Self::#option_fn_names))*
                }

                fn parse(
                    options: Vec<serenity_commands::serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption>
                ) -> std::result::Result<Self, serenity_commands::error::ParseError> {
                    use serenity_commands::serenity::model::interactions::application_command::{ApplicationCommandOptionType, ApplicationCommandInteractionDataOptionValue};

                    #(let mut #option_idents = None;)*

                    for opt in options {
                        match &opt.name[..] {
                            #(#option_names => {
                                if let Some(v) = opt.resolved {
                                    match v {
                                        ApplicationCommandInteractionDataOptionValue::#option_data_extractions => #option_idents = Some(v),
                                        _ => {
                                            return Err(serenity_commands::error::ParseError::InvalidType(
                                                ApplicationCommandOptionType::#option_kinds
                                            ));
                                        },
                                    };
                                }
                            }),*
                            s => return Err(serenity_commands::error::ParseError::UnknownOption(s.to_string())),
                        }
                    }

                    #(let #option_idents = #option_idents.ok_or(serenity_commands::error::ParseError::MissingOption(#option_names))?;)*

                    Ok(Self { #(#option_idents),* })
                }

                fn parse_command(
                    data: serenity_commands::serenity::model::interactions::application_command::ApplicationCommandInteractionData
                ) -> std::result::Result<Self, serenity_commands::error::ParseError> {
                    if data.name != Self::name() {
                        return Err(serenity_commands::error::ParseError::UnknownCommand(data.name.clone()));
                    }

                    Self::parse(data.options)
                }

                fn parse_subcommand(
                    option: serenity_commands::serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption
                ) -> std::result::Result<Self, serenity_commands::error::ParseError> {
                    if option.name != Self::name() {
                        return Err(serenity_commands::error::ParseError::UnknownSubCommand(option.name.clone()));
                    }

                    Self::parse(option.options)
                }
            });
        }

        extra
    };

    let output = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            fn name() -> &'static str {
                #cmd
            }

            fn description() -> &'static str {
                #description
            }

            #extra
        }
    };

    Ok(output)
}

fn generate_option_registration_fn(opt: &CommandOption, tokens: &mut TokenStream) -> Ident {
    let CommandOption {
        ident,
        name,
        description,
        kind,
        required,
    } = opt;

    let name = name.clone().unwrap_or_else(|| ident.to_string());
    let description = description.clone().unwrap();

    let fn_name = format_ident!("register_option_{}", ident);

    let extra = {
        let mut extra = TokenStream::new();

        if *required {
            extra.extend(quote! {
                opt.required(true);
            });
        }

        extra
    };

    tokens.extend(quote! {
        fn #fn_name(
            opt: &mut serenity_commands::serenity::builder::CreateApplicationCommandOption
        ) -> &mut serenity_commands::serenity::builder::CreateApplicationCommandOption {
            use serenity_commands::serenity::model::interactions::application_command::ApplicationCommandOptionType;

            opt.name(#name).description(#description).kind(ApplicationCommandOptionType::#kind);

            #extra

            opt
        }
    });

    fn_name
}
