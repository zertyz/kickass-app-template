//! Home for all frontends & UIs

pub mod console;
pub mod terminal;
pub mod egui;
pub mod telegram;
pub mod web;

use crate::frontend::egui::Egui;


pub enum AvailableFrontends {
    Console,
    Terminal,
    Egui,
}

pub fn run(frontend: AvailableFrontends) {
    match frontend {
        AvailableFrontends::Console => {
            console::run();
        },
        AvailableFrontends::Terminal => {
            terminal::run();
        }
        AvailableFrontends::Egui => {
            Egui::run_egui_app(format!("We are!!"), 5.1);
        }
    };
}