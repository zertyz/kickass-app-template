use std::{
    ops::RangeInclusive,
    ffi::OsStr,
};
use structopt::{StructOpt};
use strum::{EnumString, AsRefStr, VariantNames, EnumVariantNames};
use chrono::NaiveDate;
use crate::config::config_model::*;


pub fn parse_from_args() -> CommandLineOptions {
    CommandLineOptions::from_args()
}

/// merges the higher priority command line options with the application-wide config (which, most probably, came from parsing the configuration file),
/// returning a new, merged, application-wide config or panicking, if there are inconsistencies
pub fn merge_config_file_and_command_line_options(mut config_file_options: Config, command_line_options: CommandLineOptions) -> Config {

    // logging
    //////////

    // --quiet causes logging to be disabled, unless we're logging to a file
    if command_line_options.quiet && command_line_options.log_to_file.is_none() {
        if let LoggingOptions::ToFile{..} = config_file_options.log {
            // noop
        } else {
            config_file_options.log = LoggingOptions::Quiet;
        }
    }

    if let Some(_file) = command_line_options.log_to_file {
        // TODO 2022-05-12: logging to file currently requires providing all the rotation & stuff info
        //                  or we may interpret passing this option as just overriding the file name... having an effect only if config is already telling to log to a file
        //config_file_options.log =
    }

    // ui
    /////
    match command_line_options.ui {
        Ui::Console(_) => config_file_options.ui = UiOptions::Console,  // app config must handle the console command to run
        Ui::Terminal   => config_file_options.ui = UiOptions::Terminal,
        Ui::Egui       => config_file_options.ui = UiOptions::Egui,
    }
    // check: Terminal UI and Console Logging are incompatible
    if let UiOptions::Terminal = config_file_options.ui {
        if let LoggingOptions::ToConsole = config_file_options.log {
            panic!("while merging file config & command line options: It is not possible to run the Terminal UI while logging to console. Either log to a file (or disable logging) OR choose another UI");
        }
    }

    config_file_options
}

/// Command-line options
#[derive(Debug,StructOpt)]
#[structopt(about = "
================================================================
Here you should add a brief description of what this executable
does. Be as succinct as possible, but no more succinct than that.
Default & advanced options are in ${0}.config. Some of them may
be overridden by the command-line options bellow:
================================================================
")]
pub struct CommandLineOptions {

    // FLAGS
    ////////

    /// Suppresses all output to stdout and stderr
    #[structopt(long)]
    quiet: bool,

    /// Sends all output to the given file
    #[structopt(long)]
    log_to_file: Option<String>,


    // FRONT END / UI COMMANDS
    //////////////////////////

    /// Forcibly runs one of the available UIs, instead of using automatic detection
    #[structopt(subcommand)]
    pub ui: Ui,

    // /// forcibly runs the application's console UI, but suppresses all logging output -- nothing will go to stdout, stderr nor to any logging file
    // #[structopt(long)]
    // quiet: bool,
    //
    // /// forcibly runs the application's console UI, but sends all output to the given file -- which is never rotated
    // #[structopt(long, parse(from_os_str))]
    // log_to_file: Option<Stromg>,
    //
    // /// forcibly runs the application's console UI, sending all logs to stdout
    // console: bool,
    //
    // /// forcibly runs the application's EGui UI (even if no graphics environment has been detected)
    //
    // /// Searches for input raw data files in the provided path instead of the default
    // #[structopt(long, parse(from_os_str))]
    // raw_input_dir: Option<PathBuf>,
    //
    // /// Searches for dataset's raw data in the provided path instead of the default
    // #[structopt(long, parse(from_os_str))]
    // rkyv_output_dir: Option<PathBuf>,
    //
    // /// Specifies which datasets to check for updates & generate -- All, AllOfPulsePoint, NpiFullData, AllOfB3, Trades, HistoricalQuotes
    // #[structopt(short, long, default_value = DatasetSelection::AllOfB3.as_ref())]
    // dataset_selection: DatasetSelection,
    //
    // /// Selects between parallel or standard algorithms for datasets reading multiple files. Possible values: Off, On
    // /// -- n_tasks = 0 will use the same number of tasks as there are CPUs.
    // #[structopt(short, long, default_value = configs::ParallelizationOption::On{n_tasks:0}.as_ref())]
    // parallelization: configs::ParallelizationOption,
    //
    // /// Reprocess the selected datasets even if the raw data appears not to have changed (controlled by IN/OUT file's timestamps)
    // #[structopt(short, long)]
    // force: bool,
    //
    // /// If specified, whenever a dataset is opened by this program (either by the tests after it's generation or when --only-check is given),
    // /// load all their data (into OS buffers) in the process of validating it's SHA-512 hash
    // #[structopt(long)]
    // preload_and_checksum: bool,
    //
    // /// Instead of processing & generating the datasets, simply shows the dataset's models (Rust source file) this executable would generate
    // #[structopt(long)]
    // _only_models: bool,
    //
    // /// Instead of processing & generating the datasets, open the existing ones and execute a small sanity check on them
    // #[structopt(long)]
    // only_check: bool,
    //
    // /// Advanced option to select a date range other than the default -- tipically it only makes sense if used with a specific `--dataset_selection`
    // #[structopt(long, parse(from_os_str = range_inclusive_date_from_str))]
    // period: Option<RangeInclusive<NaiveDate>>,

}

/// for docs, please see description in [CommandLineOptions::Ui]
#[derive(Debug,StructOpt)]
pub enum Ui {
    /// Forcibly runs the application's console UI -- `run ${0} console --help` for more details
    Console(ConsoleSubCommands),
    /// Forcibly runs the application's Terminal UI (even if no terminal has been detected)
    Terminal,
    /// Forcibly runs the application's EGui UI (even if no graphics environment has been detected)
    Egui,
}

/// for docs, see [Ui::Console]
#[derive(Debug,StructOpt)]
pub enum ConsoleSubCommands {
    /// Service: Executes Operation A until asked to quit via a SIGTERM signal
    OperationA {
        /// Exemplary option from an enum --
        /// selects between parallel or standard algorithms.
        #[structopt(short, long, possible_values = ParallelizationOptions::VARIANTS)]
        parallelization: ParallelizationOptions,
        /// Exemplary option on how to inform a floating point number
        #[structopt(long)]
        float: Option<f64>,
        /// Advanced option to exemplify how to inform a date range -- in the form 'YYYY-MM-DD..=YYYY-MM-DD'
        #[structopt(long, parse(from_os_str = range_inclusive_date_from_str))]
        period: Option<RangeInclusive<NaiveDate>>,
    },
    /// Job: Outputs the sanity check script for this application
    SanityCheck,
}

/// maps [crates::config::ParallelizationOptions] to command line options
/// -- for docs, see [ConsoleSubCommands::OperationA::parallelization]
#[derive(Debug,StructOpt,EnumString,AsRefStr,EnumVariantNames)]
#[strum(serialize_all = "PascalCase")]
pub enum ParallelizationOptions {
    Off,
    On,
}


/// parses `arg`s in the form "2022-03-16..=2022-03-31"
fn range_inclusive_date_from_str(arg: &OsStr) -> RangeInclusive<NaiveDate> {
    let mut split = arg.to_str().unwrap().split("..=");
    let mut parse_next_date = || match split.next() {
        Some(str_date) => NaiveDate::parse_from_str(str_date, "%Y-%m-%d")
            .map_err(|err| format!("Could not parse date '{}' from RangeInclusive dates '{:?}': {}", str_date, arg, err))
            .unwrap(),
        None => panic!("{:?} is not a RangeInclusive<NaiveDate>. Inclusive Ranges are in the form \"start..=finish\"", arg)
    };
    let first_date = parse_next_date();
    let last_date = parse_next_date();
    first_date ..= last_date
}
