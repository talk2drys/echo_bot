use log;
use tokio;
use std::env;
use env_logger;
use serenity::{
    async_trait,
    model::{ gateway::Ready, channel::Message },
    client::{ Client, Context, EventHandler }
};

#[tokio::main]
async fn main() {
    // initialize logger
    env_logger::init();

    let user_discord_token: String;

    // for security reason read  token from process environment
    match env::var("USER_TOKEN") {
        Ok(token) => {
            user_discord_token = token;
            println!("{}", user_discord_token);
            log::info!("User dicord token found");
        },
        Err(var_error) => {
            match var_error {
                env::VarError::NotPresent => {
                    log::error!("User token not found");
                    std::process::exit(1);
                },
                env::VarError::NotUnicode(_) => {
                    log::error!("User token value not a unicode OSString");
                    std::process::exit(1);
                }
            }
        }
    }

    // create a client instance that would connect to discord as you the user
    let mut self_bot = Client::builder(&user_discord_token)
        .event_handler(ClientHandler).await
        .expect("Err creating client");

    // start bot
    self_bot.start().await;
}


/// Handler for the client bot
struct ClientHandler;

#[async_trait]
impl EventHandler for ClientHandler {
    // client bot message event handler, this handler receives all messages sent to
    // all guilds and channels client is a member of.
    async fn message(&self, _ctx: Context, message: Message) {
        // for now we will just log the messages to the screen
        log::info!("Message received {}", message.content);
    }

    // fires when bot is connected
    async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {
        log::info!("Client bot successfully logged in as user")
    }
}
