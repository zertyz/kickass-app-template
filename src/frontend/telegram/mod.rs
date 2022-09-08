//! Telegram UI for our ogre-datasets-converter application.
//! Contains the following UIs:
//!   * A reporter of `update` & `query` console commands -- sending messages with their statuses / output to the telegram bot
//!   * A service able to read `Query` commands (mapped from the command line options) from telegram messages
//!     -- please refer to [crate::command_line] for info on the `Query` command

mod telegram;
pub use telegram::*;