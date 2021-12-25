use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::*;

mod parse;

use parse::*;

pub fn derive_commands(item: TokenStream) -> Result<TokenStream> {
    let input = parse2::<DeriveInput>(item)?;

    let Commands {
        commands,
    } = parse_commands(&input)?;

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let command_vars = commands
        .iter()
        .map(|s| format_ident!("{}", s.to_string().to_lowercase()))
        .collect::<Vec<_>>();

    let output = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            fn register_commands(
                cmds: &mut serenity_commands::serenity::builder::CreateApplicationCommands
            ) -> &mut serenity_commands::serenity::builder::CreateApplicationCommands {
                #(cmds.create_application_command(#commands::register_command);)*
                cmds
            }

            pub(crate) async fn register_commands_globally(ctx: &serenity_commands::serenity::client::Context) {
                use serenity_commands::serenity::model::interactions::application_command::ApplicationCommand;
                ApplicationCommand::set_global_application_commands(ctx, Self::register_commands)
                    .await
                    .unwrap();
            }

            pub(crate) async fn register_commands_in_guild(
                ctx: &serenity_commands::serenity::client::Context,
                guild_id: serenity_commands::serenity::model::id::GuildId,
            ) {
                guild_id.set_application_commands(ctx, Self::register_commands)
                    .await
                    .unwrap();
            }

            pub(crate) fn parse(
                interaction: serenity_commands::serenity::model::interactions::application_command::ApplicationCommandInteraction
            ) -> std::result::Result<Self, serenity_commands::error::ParseError> {
                #(let #command_vars = #commands::name();)*

                match &interaction.data.name[..] {
                    #(s if s == #command_vars => #commands::parse_command(interaction.data).map(Self::#commands),)*
                    s => Err(serenity_commands::error::ParseError::UnknownCommand(s.to_string())),
                }
            }
        }
    };

    Ok(output)
}
