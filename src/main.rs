use env_logger;
use log;
use std::path::Path;
use serde::Deserialize;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    http,
    model::{channel::Message, gateway::Ready, id::ChannelId},
};
use std::collections::HashMap;
use std::{fs, env};
use tokio::{
    self,
    sync::mpsc,
};
use toml;
use std::sync::Arc;

#[derive(Debug, Deserialize, Default)]
struct Config {
    client_token: String,
    bot_token: String,
    mappings: HashMap<String, String>,
}

#[tokio::main]
async fn main() {
    // initialize logger
    env_logger::init();

    // would prefer to leave this uninitialized
    let mut config: std::sync::Arc<Config> = Arc::new(Config::default());
    // let user_discord_token: String;
    let config_path = Path::new("./conf/echo.toml");


    match fs::read_to_string(&config_path) {
        Ok(ref config_string) => {
            // parse the value
            if let Ok(conf) = toml::from_str::<Config>(&config_string) {
                config = Arc::new(conf);
            } else {
                // TODO: catch invalid config errors
                std::process::exit(-1);
            };
        },
        Err(err) => {
            println!("Error reading string:: {:?}", err);
        },
    };

    // channel for client and  bot instances to communicate
    let (tx, rx) = mpsc::channel::<BotMessage>(100);

    let client_h = ClientHandler { tx };

    // create a client instance that would connect to discord as you the user
    let self_bot = Client::builder(&config.client_token)
        .event_handler(client_h)
        .await
        .expect("Err creating client");

    let client_handle = tokio::spawn(run_client(self_bot));
    let conff = config.clone();

    let bot_handle = tokio::spawn(
        start_bot(conff.bot_token.clone(), rx, conff.mappings.clone())
    );

    client_handle.await;
    bot_handle.await;

    println!("The end");
}

/// Handler for the client bot
struct ClientHandler {
    tx: mpsc::Sender<BotMessage>,
}

#[async_trait]
impl EventHandler for ClientHandler {
    // client bot message event handler, this handler receives all messages sent to
    // all guilds and channels client is a member of.
    async fn message(&self, _ctx: Context, message: Message) {
        // check channels message is coming f
        // log::info!("Message received {}", message.content);
        if let Err(err) = &self.tx.send( BotMessage{ channel_id: message.channel_id.0, message: message.content, }).await {
            log::warn!("Error sending message {} to channel {}", err.0.message, err.0.channel_id);
        }
    }

    // fires when bot is connected
    async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {
        log::info!("Client bot successfully logged in as user");
    }
}

/// message struct pass from client instance to bot instance
struct BotMessage {
    /// channel id bot message should be post to
    channel_id: u64,
    /// message to be posted
    message: String,
}

async fn run_client(mut client: Client) {
    client.start().await;
}

async fn start_bot<T: AsRef<str>>(
    token: T,
    mut rx: mpsc::Receiver<BotMessage>,
    channel_map: HashMap<String, String>,
) {
    while let Some(msg) = rx.recv().await {
        if let Some(chan_id) = channel_map.get(&msg.channel_id.to_string()) {
            let http_instance = http::Http::new_with_token(token.as_ref());
            
            let id = chan_id.parse::<u64>().unwrap();
            // http_instance.broadcast_typing(id).await;

            // create a new http client
            ChannelId(id)
                .say(http_instance, msg.message).await;
        }
    }
}

