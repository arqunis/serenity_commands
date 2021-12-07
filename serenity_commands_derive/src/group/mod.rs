use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::*;

mod parse;

use parse::*;

pub fn derive_group(item: TokenStream) -> Result<TokenStream> {
    let input = parse2::<DeriveInput>(item)?;

    let Group {
        name: group,
        description,
        subcommands,
    } = parse_group(&input)?;

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let extra = {
        let mut extra = TokenStream::new();

        let subcommand_vars = subcommands
            .iter()
            .map(|s| format_ident!("{}", s.to_string().to_lowercase()))
            .collect::<Vec<_>>();

        extra.extend(quote! {
            fn register_subcommand_group(
                opt: &mut serenity_commands::serenity::builder::CreateApplicationCommandOption
            ) -> &mut serenity_commands::serenity::builder::CreateApplicationCommandOption {
                use serenity_commands::serenity::model::interactions::application_command::ApplicationCommandOptionType;

                opt.name(Self::name())
                    .description(Self::description())
                    .kind(ApplicationCommandOptionType::SubCommandGroup)
                    #(.create_sub_option(#subcommands::register_subcommand))*
            }

            fn parse_subcommand_group(
                option: serenity_commands::serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption
            ) -> std::result::Result<Self, serenity_commands::error::ParseError> {
                if option.name != Self::name() {
                    return Err(serenity_commands::error::ParseError::UnknownSubCommandGroup(option.name.clone()));
                }

                #(let #subcommand_vars = #subcommands::name();)*

                for opt in option.options {
                    #(if opt.name == #subcommand_vars {
                        return Ok(Self::#subcommands(#subcommands::parse_subcommand(opt)?));
                    })*

                    return Err(serenity_commands::error::ParseError::UnknownSubCommand(opt.name.clone()));
                }

                unreachable!()
            }
        });

        extra
    };

    let output = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            fn name() -> &'static str {
                #group
            }

            fn description() -> &'static str {
                #description
            }

            #extra
        }
    };

    Ok(output)
}
