//! Basis for a telegram application
//! For an improved version, microservice the design of this SMS framework may land some ideas: https://github.com/zertyz/InstantVAS-backend-java-opensource

use crate::config::{
    config_model::{Config, TelegramConfig, TelegramBotOptions},
};
use std::{
    sync::Arc,
    pin::Pin,
};
use teloxide::{
    prelude::*,
    utils::command::BotCommands,
    dispatching::{
        dialogue::InMemStorage,
        ShutdownToken
    },
};
use log::{debug, info};

//const TELEGRAM_TOKEN: &str = "1662319489:AAFbgJyCpfLOOfxJTq8Ms_nSmKNft7Lwi5M";   // AirTicketRobots' token

/// starts a Telegram chat application, returning a shutdown callback (invoke it to initiate the shutdown sequence)
pub async fn run(config: Arc<Config>) -> Box<dyn FnOnce() -> Pin<Box<dyn std::future::Future<Output = ()>>>> {
    let shutdown_token = if let Some(telegram_config) = &config.services.telegram {
        let shutdown_token = match telegram_config.bot {
            TelegramBotOptions::Dice      => dice_bot(&telegram_config.token).await,
            TelegramBotOptions::Stateless => stateless_commands(&telegram_config.token).await,
            TelegramBotOptions::Stateful  => stateful_commands(&telegram_config.token).await,
        };
        Some(shutdown_token)
    } else {
        None
    };

    // TODO 2022-05-13: redesign this while closure returning, as the returned one is only available when the application exits
    // the shutdown FnOnce():
    Box::new(|| Box::pin( async {
        if let Some(shutdown_token) = shutdown_token {
            debug!("Informing Teloxide (Telegram) of the shutdown intention");
            shutdown_token.shutdown().expect("Teloxide (Telegram) refused to shutdown").await;
        }
    }))
}


async fn dice_bot(token: &str) -> ShutdownToken {
    info!("Starting throw dice bot...");

    let bot = Bot::new(token).auto_send();

    let _handler = |message: Message, bot: AutoSend<Bot>| async move {
        bot.send_dice(message.chat.id).await?;
        respond(())
    };

    let ignore_update = |_upd| Box::pin(async {});
    let listener = teloxide::dispatching::update_listeners::polling_default(bot.clone()).await;

    let mut dispatcher = Dispatcher::builder(bot.clone(), Update::filter_message().chain(dptree::endpoint(handler)))
        .default_handler(ignore_update)
        .build();
    dispatcher
        .setup_ctrlc_handler()
        .dispatch_with_listener(
        listener,
        LoggingErrorHandler::with_custom_text("An error from the update listener"),
    ).await;

    /// handler for the bot messages
    async fn handler(message: Message, bot: AutoSend<Bot>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.send_dice(message.chat.id).await?;
        Ok(())
    }

    dispatcher.shutdown_token()
}


async fn stateless_commands(token: &str) -> ShutdownToken {
    info!("Starting stateless command bot...");

    let bot = Bot::new(token).auto_send();

    let ignore_update = |_upd| Box::pin(async {});
    let listener = teloxide::dispatching::update_listeners::polling_default(bot.clone()).await;

    let mut dispatcher = Dispatcher::builder(bot.clone(), Update::filter_message().filter_command::<Commands>().chain(dptree::endpoint(handler)))
        .default_handler(ignore_update)
        .build();
    dispatcher
        .setup_ctrlc_handler()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener")
        ).await;


    // commands this bot accepts -- start by sending it '/help'
    #[derive(BotCommands, Clone)]
    #[command(rename = "lowercase", description = "These commands are supported:")]
    enum Commands {
        #[command(description = "display this text")]
        Help,
        #[command(description = "informs your desired username")]
        Username(String),
        #[command(description = "informs your desired username and and age", parse_with = "split")]
        UsernameAndAge { username: String, age: u8 },
    }

    /// handler for the bot messages
    async fn handler(bot: AutoSend<Bot>, message: Message, command: Commands) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match command {
            Commands::Help => {
                bot.send_message(message.chat.id, Commands::descriptions().to_string()).await?;
            }
            Commands::Username(username) => {
                bot.send_message(message.chat.id, format!("Your username is @{username}.")).await?;
            }
            Commands::UsernameAndAge { username, age } => {
                bot.send_message(message.chat.id, format!("Your username is @{username} and age is {age}.")).await?;
            }
        }
        Ok(())
    }

    dispatcher.shutdown_token()
}


async fn stateful_commands(token: &str) -> ShutdownToken {
    type MyDialogue = Dialogue<State, InMemStorage<State>>;
    type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    info!("Starting dialogue bot...");

    let bot = Bot::new(token).auto_send();

    let mut dispatcher = Dispatcher::builder(bot.clone(), Update::filter_message().enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::ReceiveFullName].endpoint(receive_full_name))
            .branch(dptree::case![State::ReceiveAge { full_name }].endpoint(receive_age))
            .branch(dptree::case![State::ReceiveLocation { full_name, age }].endpoint(receive_location))
    )
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .build();
    dispatcher
        .setup_ctrlc_handler()
        .dispatch().await;

    #[derive(Clone)]
    pub enum State {
        Start,
        ReceiveFullName,
        ReceiveAge { full_name: String },
        ReceiveLocation { full_name: String, age: u8 },
    }

    impl Default for State {
        fn default() -> Self {
            Self::Start
        }
    }

    async fn start(bot: AutoSend<Bot>, msg: Message, dialogue: MyDialogue) -> HandlerResult {
        bot.send_message(msg.chat.id, "Let's start! What's your full name?").await?;
        dialogue.update(State::ReceiveFullName).await?;
        Ok(())
    }

    async fn receive_full_name(
        bot: AutoSend<Bot>,
        msg: Message,
        dialogue: MyDialogue,
    ) -> HandlerResult {
        match msg.text() {
            Some(text) => {
                bot.send_message(msg.chat.id, "How old are you?").await?;
                dialogue.update(State::ReceiveAge { full_name: text.into() }).await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Send me plain text.").await?;
            }
        }

        Ok(())
    }

    async fn receive_age(
        bot: AutoSend<Bot>,
        msg: Message,
        dialogue: MyDialogue,
        full_name: String, // Available from `State::ReceiveAge`.
    ) -> HandlerResult {
        match msg.text().map(|text| text.parse::<u8>()) {
            Some(Ok(age)) => {
                bot.send_message(msg.chat.id, "What's your location?").await?;
                dialogue.update(State::ReceiveLocation { full_name, age }).await?;
            }
            _ => {
                bot.send_message(msg.chat.id, "Send me a number.").await?;
            }
        }

        Ok(())
    }

    async fn receive_location(
        bot: AutoSend<Bot>,
        msg: Message,
        dialogue: MyDialogue,
        (full_name, age): (String, u8), // Available from `State::ReceiveLocation`.
    ) -> HandlerResult {
        match msg.text() {
            Some(location) => {
                let message = format!("Full name: {full_name}\nAge: {age}\nLocation: {location}");
                bot.send_message(msg.chat.id, message).await?;
                dialogue.exit().await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Send me plain text.").await?;
            }
        }

        Ok(())
    }

    dispatcher.shutdown_token()
}