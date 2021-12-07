use serenity_commands::serenity;
use serenity_commands::macros::{Command, Commands};

use serenity::client::{Client, Context, EventHandler};
use serenity::model::interactions::*;
use serenity::model::prelude::*;

/// Play a little game called Ping Pong!
#[derive(Debug, Command)]
#[command(name = "ping")]
struct Ping {
    /// Amount of pings to send.
    #[option(integer, required)]
    n: i64,
}

#[derive(Commands)]
enum Command {
    Ping(Ping),
}

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        Command::register_commands_globally(&ctx).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let interaction = match interaction {
            Interaction::ApplicationCommand(cmd) => cmd,
            _ => return,
        };

        let command = match Command::parse(interaction.clone()) {
            Ok(cmd) => cmd,
            Err(_) => return,
        };

        match command {
            Command::Ping(Ping { n }) => {
                interaction
                    .create_interaction_response(&ctx, |r| {
                        r.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|m| m.content(format!("Pong {} times!", n)))
                    })
                    .await
                    .unwrap();
            },
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("DISCORD_TOKEN")?;
    let application_id = std::env::var("APPLICATION_ID")?;
    let application_id = application_id.parse::<u64>()?;

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .application_id(application_id)
        .await?;

    client.start_autosharded().await?;

    Ok(())
}
