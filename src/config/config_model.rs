
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
    /// Specifies what the application should do with it's log messages
    pub log: LoggingOptions,
    /// Specifies what parallelization policy applicable algorithms should use
    pub parallelization: ParallelizationOptions,
    /// Services (and their configs) to be enabled
    pub services: ServicesConfig,
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

#[derive(Debug,Serialize,Deserialize)]
pub struct ServicesConfig {
    pub telegram: Option<TelegramConfig>,
    pub web:      Option<WebConfig>,
}

/// The telegram service
#[derive(Debug,Serialize,Deserialize)]
pub struct TelegramConfig {
    /// Telegram's bot token, obtained from "BotFather's" bot:
    /// 1) Open TelegramApp and search for BotFather
    /// 2) Send /newbot (or /help)
    pub token: String,
    /// The bot to use
    pub bot: TelegramBotOptions,
}

/// The HTTP/HTTPS service
#[derive(Debug,Serialize,Deserialize)]
pub struct WebConfig {
    /// The Rocket profile to use as basis for `rocket_config`
    pub profile: RocketProfiles,
    /// Rocket config details
    pub rocket_config: RocketConfigOptions,
    /// If set, enables [crate::frontend::web::sanity_check] routes -- allowing this executable to be probed for it's running sanity
    pub sanity_check_routes: bool,
    /// If set, enables [crates::frontend::web::stats] routes -- exposing runtime metrics
    pub stats_routes: bool,
    /// If set, enables [crates::frontend::web::logs_following] routes -- exposing online logs for the app
    pub logs_following_routes: bool,
    /// If set, enables [crates::frontend::web::ogre_events_following] routes -- exposing online `Ogre Events` for the app
    pub ogre_events_following_routes: bool,
    /// If set, enables [crates::frontend::web::ogre_events_queue] routes -- exposing `Ogre Events` designed to be consumed by external services
    pub ogre_events_queue_routes: bool,
    /// If set, enables the Angular application present in `web-app/`, exposing it's [crate::frontend::web::backend]
    /// routes and all related static files (see [crate::frontend::web::embedded_files])
    pub web_app: bool,
    /// Prepends the given string to all our HTTP/HTTPS routes
    pub routes_prefix: String,
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

/// Rocket requires us to inform in which "environment" we're running.\
/// If you use the [RocketConfigOptions::StandardRocketTomlFile] variant, each section
/// must be present on the file.
#[derive(Debug,Serialize,Deserialize)]
pub enum RocketProfiles {
    Debug,
    Production,
}

/// Available Rocket configuration possibilities
#[derive(Debug,Serialize,Deserialize)]
pub enum RocketConfigOptions {
    /// Instructs Rocket to read configs from it's `Rocket.toml` file. Notice that Rocket will look
    /// for such file in the current working directory, rather than on the executable's location.
    StandardRocketTomlFile,
    /// When using use only HTTP, using this variant may be desireable, as it avoids the need of managing
    /// another configuration file: `Rocket.toml` -- Rocket's config.
    Provided {
        /// Port to listen to HTTP connections
        http_port:  u16,
        /// How many tokio async tasks should be used to process the incoming requests?
        workers: u16,
    }
}

/// Logging options -- what to do with log messages
#[derive(Debug,Serialize,Deserialize)]
pub enum LoggingOptions {
    /// Simply ignore them
    Quiet,
    /// Output them to stdout
    ToConsole,
    /// Save them to the specified file, with the specified options:
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
            log:             LoggingOptions::ToConsole,
            parallelization: ParallelizationOptions::On{n_tasks: 0},
            services:        ServicesConfig {
                telegram: Some(TelegramConfig {
                    token: String::from("<<Open TelegramApp, search for BotFather, send /newbot>>"),
                    bot: TelegramBotOptions::Dice,
                }),
                web: Some(WebConfig {
                    profile: RocketProfiles::Debug,
                    rocket_config: RocketConfigOptions::Provided {
                        http_port: 8000,
                        workers:   1,
                    },
                    sanity_check_routes:          false,
                    stats_routes:                 false,
                    logs_following_routes:        false,
                    ogre_events_following_routes: false,
                    ogre_events_queue_routes:     false,
                    web_app:                      true,
                    routes_prefix: "".to_string()
                }),
            },
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
