//! Application-wide configuration.
//!
//! The contents of [config_model::Config] are meant to be shared between [frontend] and all business logic components.\
//! This module is able to load & save configuration values from/to a given file -- see it's usage in [main].\
//! ... and there is a contract with [crate::command_line], assuring flags specified there have a higher priority over the ones
//! specified on the configuration file.

pub mod config_model;
pub use config_model::Config;
use std::fs;
use regex::Regex;


/// transcription of the config model, for documentation purposes when writing the default config file
const CONFIG_MODEL_DOCS: &str = include_str!("config_model.rs");


/// loads the application-wide configuration from the given `config_file_path`
/// or create it (with default values) if it doesn't exist
pub fn load_or_create_default(config_file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let result = load_from_file(config_file_path);
    if result.is_ok() {
        result
    } else {
        let error_message = result.as_ref().unwrap_err().to_string();
        let default_config = Config::default();
        if error_message.contains("No such file or directory") {
            save_to_file(&default_config, config_file_path)?;
            Ok(default_config)
        } else {
            result
        }
    }
}

/// loads the application-wide configuration from the given `config_file_path`, if possible
fn load_from_file(config_file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let ron_file_contents = fs::read_to_string(config_file_path)?;
    ron::from_str(&ron_file_contents)
        .map_err(|err| Box::try_from(format!("config.rs: Error deserializing contents of file '{}' as RON: {}", config_file_path, err)).unwrap())
}

/// saves the application-wide `config` to `config_file_path`,
/// including documentation from the original [config_model] sources
fn save_to_file(config: &Config, config_file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut extensions = ron::extensions::Extensions::all();
    extensions.remove(ron::extensions::Extensions::IMPLICIT_SOME);
    let data_section = ron::ser::to_string_pretty(
        &config,
        ron::ser::PrettyConfig::new()
            .depth_limit(10)
            .new_line(String::from("\n"))
            .indentor(String::from("    "))
            .separate_tuple_members(true)
            .enumerate_arrays(true)
            .decimal_floats(true)
            .extensions(extensions))
        .map_err(|err| format!("config.rs: Error serializing config as TOML: {}", err))?;

    // include documentation on the written file, with the Regex replacements declared there
    let docs_section = config_model::REPLACEMENTS.iter()
        .fold(String::from(CONFIG_MODEL_DOCS), |s, (from, to)| {
            let regex = Regex::new(from).expect("Error parsing regex");
            regex.replace_all(&s, *to).to_string()
        });

    let config_file_contents = format!("{}\n\n/*{}*/\n", data_section, docs_section);

    fs::write(config_file_path, config_file_contents)
        .map_err(|err| Box::try_from(format!("config.rs: Error writing default RON config to file '{}': {}", config_file_path, err)).unwrap())
}


/// Unit tests the [config](self) module
#[cfg(any(test, feature = "dox"))]
mod tests {
    use rocket::form::validate::Contains;
    use super::*;

    const TEST_CONFIG_FILE: &str = "/tmp/kickass-app-template-tests.config.ron";

    #[cfg_attr(not(feature = "dox"), test)]
    fn file_load_and_save() {
        fs::remove_file(TEST_CONFIG_FILE).unwrap_or(());

        // load non existing file
        let result = load_from_file(TEST_CONFIG_FILE);
        println!("Loading from file: '{:?}'", result);
        assert!(result.is_err(), "Loading from an non existing file should have returned an error");
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("No such file or directory"), "Error message '{}' does not contain 'No such file or directory'", error_message);

        // save to non existing file
        save_to_file(&Config::default(), TEST_CONFIG_FILE)
            .expect("Could not save config file");

        // load existing file
        let _result = load_from_file(TEST_CONFIG_FILE)
            .expect("Could not load config file");

        // check load_or_create_default() for existing file
        let _result = load_or_create_default(TEST_CONFIG_FILE)
            .expect("Could not load_or_create_default() for an existing file");

        fs::remove_file(TEST_CONFIG_FILE).unwrap_or(());

        // check load_or_create_default() for non existing file
        let _result = load_or_create_default(TEST_CONFIG_FILE)
            .expect("Could not load_or_create_default() for a non existing file");

    }

}