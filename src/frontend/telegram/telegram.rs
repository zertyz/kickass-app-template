//! see [super]

use crate::config::{Config, TelegramConfig, TelegramBotOptions};
use std::{
    sync::Arc,
    borrow::{Borrow, Cow},
};
use owning_ref::OwningRef;
use futures::{
    SinkExt,
    future::BoxFuture
};
use teloxide::{
    prelude::*,
    utils::command::BotCommands,
    dispatching::{
        ShutdownToken
    },
    dispatching::{
        DefaultKey,
        dialogue::InMemStorage,
    },
};
use log::debug;


/// prefix to all debug log messages, so to better contextualize them
const DEBUG_IDENT: &str = "      ";


/// Returned by this module when the Telegram UI starts -- see [runner()].\
/// Use to, programmatically, interact with the Telegram UI:
///  * inquire if there is a service running able to answer to MO (Mobile Originated) messages;
///  * inquire if sending MTs (Mobile Terminated) are allowed;
///  * request the UI service to shutdown.
pub struct TelegramUI {
    /// runtime configs for our UI service
    telegram_config: OwningRef<Arc<Config>, TelegramConfig>,
    /// Teloxide's bot
    bot: AutoSend<Bot>,
    /// Teloxide's dispatcher associated with [bot]
    dispatcher: Option<Dispatcher<AutoSend<Bot>, Box<dyn std::error::Error + Sync + Send>, DefaultKey>>,
    /// if present, exposes the Teloxide's `shutdown_token` through which one may request the service to cease running
    pub shutdown_token: Option<ShutdownToken>,
    /// if set, may be used to send MTs to the Telegram Bot
    _mt_hande: Option<bool>,
}

impl TelegramUI {

    /// call this foe with:
    /// ```no_compile
    ///     if let ExtendedOption::Enabled(telegram_config) = &config.telegram {
    ///         let shareable_telegram_controller = TelegramUI::new(telegram_config);
    ///         // this hypothetical function holds an Arc reference to the controller, from which any thread may request shutdown at any time
    ///         report_telegram_controller(shareable_telegram_controller.clone());
    ///         // the next call will run the service and block until shutdown is requested from elsewhere
    ///         shareable_telegram_controller.run_service().await?;
    ///         info!("Telegram service is DONE");
    ///     }
    pub async fn new(telegram_config: OwningRef<Arc<Config>, TelegramConfig>) -> Self {
        debug!("{}Instantiating 'teloxide' for bot token '{}'", DEBUG_IDENT, telegram_config.token);
        let bot = Bot::new(&telegram_config.token).auto_send();
        let mut instance = Self {
            telegram_config,
            bot,
            dispatcher:     None,
            shutdown_token: None,
            _mt_hande:       None,
        };
        instance.setup_bot().await;
        instance
    }

    /// sends the `message` to all registered "chat ids"
    pub async fn broadcast_message(&self, message: &str, html: bool) -> Result<(), Box<dyn std::error::Error>> {
        for chat_id in &self.telegram_config.notification_chat_ids {
            self.send_message(*chat_id, message, html).await?;
        }
        Ok(())
    }

    /// sends the `message` to the single `chat_id`
    pub async fn send_message(&self, chat_id: i64, message: &str, html: bool) -> Result<(), Box<dyn std::error::Error>> {
        // TODO 2022-11-20 Maybe an API redesign should be done for the sake of efficiency: 'adjust_message(&str) -> &[Cow<&str>]' might be introduced
        //                 to avoid the need of doing the following every time, in which case, this method should be reverted back to just sending
        //                 the message. PS: `broadcast_message()` might be one example of a function calling adjust_message() and then send_message()
        //                 as many times as needed. Note the bellow version only cuts the message and discards the rest of it, while on the proposed
        //                 'adjust_message()', we'd split it into several parts. HTML would still be a challenge...
        // adjust the message to telegram limits
        const TELEGRAM_MAX_MESSAGE_SIZE: usize = 4096;
        let mut message = Cow::Borrowed(message);
        if message.len() > TELEGRAM_MAX_MESSAGE_SIZE {
            // if the message is too big, cuts it down for sending, adding the '...' suffix to indicate there was a cut:
            // for plain text, just add it; for HTML, preserve the last closing HTML tag as well, in order not to defecate formatting
            let cutting_suffix = if !html {
                format!("...")
            }  else {
                let last_closing_tag_pos = message.rfind("</").unwrap_or(message.len());
                format!("...{}", &message[last_closing_tag_pos..])
            };
            message = Cow::Owned(format!("{}{}", &message[0..TELEGRAM_MAX_MESSAGE_SIZE -cutting_suffix.len()], cutting_suffix));
        }

        let sender = self.bot.send_message::<ChatId, &str>(teloxide::types::ChatId(chat_id), message.borrow());
        let result = if html {
            sender.parse_mode(teloxide::types::ParseMode::Html)
                .send().await
        } else {
            sender.send().await
        };
        result.map_err(|err| format!("TelegramUI: error sending push message '{}' to #{}: {}", message, chat_id, err))?;
        Ok(())
    }

    /// returns a runner, which you may call to run the telegram UI and that will only return when
    /// the service is over -- this special semantics allows holding the mutable reference to `self`
    /// as little as possible.\
    /// Example:
    /// ```no_compile
    ///     self.runner()().await;
    pub fn runner<'r>(&mut self) -> impl FnOnce() -> BoxFuture<'r, ()> + 'r {
        let bot = self.bot.clone();
        let dispatcher = self.dispatcher.take();
        || Box::pin(async move {
            if let Some(mut dispatcher) = dispatcher {
                let listener = teloxide::dispatching::update_listeners::polling_default(bot).await;
                dispatcher
                    .setup_ctrlc_handler()
                    .dispatch_with_listener(
                        listener,
                        LoggingErrorHandler::with_custom_text("An error from the update listener")
                    ).await;
            }
        })
    }

    async fn setup_bot(&mut self) {
        match self.telegram_config.bot {
            TelegramBotOptions::Dice       => (),//self.dice().await,             // returns with a Controller telling the service is not running but we're ready to send any MTs -- consider renaming this enum variant to 'SinkService'
            TelegramBotOptions::Stateless  => self.setup_query_ui_bot().await,    // starts the service able to perform query commands
            TelegramBotOptions::Stateful   => ()//self.stateful_commands().await,
        }
    }

    async fn setup_query_ui_bot(&mut self) {
        let ignore_update = |_upd| Box::pin(async {});
        let _listener = teloxide::dispatching::update_listeners::polling_default(self.bot.clone()).await;

        let dispatcher = Dispatcher::builder(self.bot.clone(), Update::filter_message().filter_command::<Commands>().chain(dptree::endpoint(handler)))
            .default_handler(ignore_update)
            .build();
        let shutdown_token = dispatcher.shutdown_token();
        self.dispatcher = Some(dispatcher);
        self.shutdown_token = Some(shutdown_token);
    }

}

// UI Business Rules
////////////////////

// commands this bot accepts -- start by sending it '/help'
// Commands (deriving BotCommands) should be based off 'TradesQueryOptions'
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
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

async fn dice_bot(token: &str) -> ShutdownToken {
    debug!("Starting throw dice bot...");

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

async fn stateful_commands(token: &str) -> ShutdownToken {
    type MyDialogue = Dialogue<State, InMemStorage<State>>;
    type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    debug!("Starting dialogue bot...");

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
