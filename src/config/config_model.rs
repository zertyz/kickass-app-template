
//! Keep, in this module, all the types used for the applications config file
//! -- the file will be placed along the `${0}.config.ron` config file to
//! serve as it's documentation

use serde::{Serialize,Deserialize};


/// CONFIG FILE DOCUMENTATION
/// (feel free to move comments & possible values close to the data)
///
/// Root for this Application's config
#[derive(Debug,Serialize,Deserialize)]
pub struct Config {
    /// The UI that should be used to run the application
    pub ui: UiOptions,
    /// Which services should be enabled?
    pub enabled_services: Vec<Services>,
    /// Specifies what the application should do with it's log messages
    pub log: LoggingOptions,
    /// Specifies what parallelization policy applicable algorithms should use
    pub parallelization: ParallelizationOptions,
}

/// UI options -- how should we present the application progress to the user?
#[derive(Debug,Serialize,Deserialize)]
pub enum UiOptions {
    /// Collects running environment data to determine the best possible Ui to use:
    /// if DISPLAY env is available, `Egui` is used, otherwise use `Terminal` if
    /// the appropriate TERM env is defined. `Console` is used as a fallback.
    Automatic,
    /// Runs the application's console UI -- run `${0} console --help` for more details
    Console,
    /// Runs the application's Terminal UI
    Terminal,
    /// Runs the application's EGui UI
    Egui,
}

/// Available services
#[derive(Debug,Serialize,Deserialize)]
pub enum Services {
    /// The telegram service
    Telegram {
        /// Telegram's bot token, obtained from "BotFather's" bot:
        /// 1) Open TelegramApp and search for BotFather
        /// 2) Send /newbot (or /help)
        token: String,
        /// The bot to use
        bot: TelegramBotOptions,
    },
    /// The HTTP/HTTPS service
    Rocket {
        /// Port to listen to HTTP connections
        http_port:  u16,
        /// Port to listen to HTTPS connections
        https_port: u16,
        /// If set, exposes the HTTP/HTTPS API at the given path -- Some("/api"), for instance
        http_api: Option<String>,
        /// If set, exposes (at the given path) the built-in Angular app + related static files
        /// & related backend services
        angular_app: Option<String>,
        /// If set, exposes (at the given path) the built-in statistics dashboard (also made in Angular) +
        /// related static files & related backend services
        stats_app: Option<String>,
        /// How many async tasks should be created to process the incoming requests?
        workers: u16,
    },
}

/// Available bots to handle Telegram interaction
#[derive(Debug,Serialize,Deserialize)]
pub enum TelegramBotOptions {
    /// Simply answers each message with a dice throw
    Dice,
    /// Only answers to known commands. Initiate a chat with this bot by sending "/help"
    Stateless,
    /// Chat-like robot, holding dialog context. Send it anything to start the conversations
    Stateful,
}

/// Logging options -- what to do with log messages
#[derive(Debug,Serialize,Deserialize)]
pub enum LoggingOptions {
    /// Simply ignore them
    Quiet,
    /// Output them to stdout
    ToConsole,
    /// Save them to the specified file
    ToFile {
        /// File to use a basis for rotation or appending
        file_path: String,
        /// The maximum size (bytes) for a log file before a rotation kicks in -- example: 1024*1024*1024 = 1073741824
        rotation_size: usize,
        /// The upper limit of rotations to keep before deleting old ones -- example: 64
        rotations_kept: usize,
        /// Performs a gzip compression after a rotation?
        compress_rotated: bool,
    },
}

/// Parallelization options for applicable algorithms
#[derive(Debug,Serialize,Deserialize)]
pub enum ParallelizationOptions {
    /// Use a non-parallel algorithms
    Off,
    /// Use a parallel algorithms with a specified limit for the total number of threads + async tasks
    On {
        /// the limit of threads + async tasks -- use 0 to have it auto-tuned for the available CPUs
        n_tasks: u16
    },
}

/////  EVERYTHING BELOW THIS LINE WILL NOT BE INCLUDED IN THE APPLICATION'S CONFIG FILE  /////

impl Default for Config {
    fn default() -> Self {
        Self {
            ui:              UiOptions::Automatic,
            enabled_services: vec![
                Services::Telegram {
                    token: String::from("<<Open TelegramApp, search for BotFather, send /newbot>>"),
                    bot: TelegramBotOptions::Dice,
                },
                Services::Rocket {
                    http_port: 8080,
                    https_port: 8888,
                    http_api: Some(String::from("/api")),
                    angular_app: Some(String::from("/app")),
                    stats_app: Some(String::from("/stats")),
                    workers: 1
                },
            ],
            log:             LoggingOptions::ToConsole,
            parallelization: ParallelizationOptions::On{n_tasks: 0},
        }
    }
}

/// Regexes and their replacements to apply to this file when writing the docs
pub const REPLACEMENTS: &[(&str, &str)] = &[
    ("\n//![^\n]*",                                                                                            ""),     // remove file doc comments
    ("\nuse serde::[^\n]*",                                                                                    ""),     // remove 'use' clause
    ("\n#[^\n]*",                                                                                              ""),     // remove macros & #[derive(...)] clauses
    ("(?s)\n/////  EVERYTHING BELOW THIS LINE WILL NOT BE INCLUDED IN THE APPLICATION'S CONFIG FILE  /////.*", ""),     // remove everything after the comment tag
    ("\n\n+",                                                                                                  "\n\n"), // standardize the number of consecutive empty lines
];
