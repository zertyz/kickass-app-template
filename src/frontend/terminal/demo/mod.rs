mod app;
//#[cfg(feature = "crossterm")]
mod crossterm;
#[cfg(feature = "termion")]
mod termion;
mod ui;

//#[cfg(feature = "crossterm")]
use self::crossterm::run;
#[cfg(feature = "termion")]
use crate::termion::run;
use std::{error::Error, time::Duration};

#[derive(Debug)]
pub struct Config {
    /// time in ms between two ticks.
    pub tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    pub(crate) enhanced_graphics: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            tick_rate:         200,
            enhanced_graphics: true,
        }
    }
}

pub fn run_demo(config: Config) -> Result<(), Box<dyn Error>> {
    let tick_rate = Duration::from_millis(config.tick_rate);
    run(tick_rate, config.enhanced_graphics)?;
    Ok(())
}
