use std::{env, time::Duration};

use dashmap::DashMap;
use serenity::{
    all::{Message, User},
    async_trait,
    prelude::*,
};
use tracing::{error, Level};

struct Handler {
    water_pending: DashMap<User, Duration>,
}
static EAU: &'static str = "https://cdn.discordapp.com/attachments/400391719910375437/1238885513390456882/Sans_titre.jpg?ex=6640e98b&is=663f980b&hm=a8601042af9b1f6b3f5898c49b7ae304bcfb3bcfdecc05545ffe5dceab27ee46&";

impl Handler {
    async fn send(&self, ctx: &Context, msg: &Message, message: &str) {
        if let Err(e) = msg.channel_id.say(&ctx.http, message).await {
            error!(?e);
            return;
        }
    }

    async fn smoke(&self, ctx: Context, msg: Message) {
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

    async fn water(&self, ctx: Context, msg: Message, mut command: Vec<String>) {
        command.remove(0);
        if command.len() < 1 {
            self.send(
                &ctx,
                &msg,
                format!("{}: Malformed command.", msg.author).as_str(),
            )
            .await;
        }

        match command[0].as_str() {
            "add" => {
                if command.len() < 2 {
                    self.send(
                        &ctx,
                        &msg,
                        format!("{}: Malformed command.", msg.author).as_str(),
                    )
                    .await;
                    return;
                }

                let parsed = command[1].parse::<u64>();

                if let Err(e) = parsed {
                    error!(?e);
                    self.send(
                        &ctx,
                        &msg,
                        format!("{}: Malformed command.", msg.author).as_str(),
                    )
                    .await;
                    return;
                }

                let minutes = parsed.unwrap();
                let duration = Duration::from_secs(minutes * 60);

                self.water_pending.insert(msg.author.clone(), duration);
                self.send(&ctx, &msg, format!("{}: :ok_hand:", msg.author).as_str())
                    .await;
                loop {
                    if self.water_pending.contains_key(&msg.author) == false {
                        return;
                    }

                    tokio::time::sleep(duration).await;

                    self.send(
                        &ctx,
                        &msg,
                        format!("{} DE L'EAU :potable_water: {}", msg.author, EAU).as_str(),
                    )
                    .await;
                }
            }
            "stop" => {
                self.water_pending.remove(&msg.author);
                self.send(&ctx, &msg, format!("{}: :ok_hand:", msg.author).as_str())
                    .await;
            }
            _ => {
                self.send(
                    &ctx,
                    &msg,
                    format!("{}: Unknown command.", msg.author).as_str(),
                )
                .await;
                return;
            }
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let command: Vec<String> = msg
            .content
            .split_whitespace()
            .map(|e| e.to_owned())
            .collect();
        if command[0].chars().next().unwrap() != '!' {
            // TODO: Error handling
            return;
        }
        match command[0].as_str() {
            "!smoke" => self.smoke(ctx, msg).await,
            "!water" => self.water(ctx, msg, command).await,
            _ => return,
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
        .event_handler(Handler {
            water_pending: DashMap::new(),
        })
        .await?;

    client.start().await?;

    Ok(())
}
