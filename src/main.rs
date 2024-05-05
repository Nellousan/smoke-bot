use std::{env, time::Duration};

use serenity::{all::Message, async_trait, prelude::*};
use tracing::{error, Level};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!smoke" {
            let minutes: u64 = (rand::random::<u64>() % 5u64) + 1;

            if let Err(e) = msg
                .channel_id
                .say(&ctx.http, format!("{} {} minutes !", msg.author, minutes))
                .await
            {
                error!(?e);
                return;
            }

            tokio::time::sleep(Duration::from_secs(minutes * 60)).await;

            if let Err(e) = msg
                .channel_id
                .say(&ctx.http, format!("{} :ok_hand:", msg.author))
                .await
            {
                error!(?e);
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_level(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN")?;

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await?;

    client.start().await?;

    Ok(())
}
