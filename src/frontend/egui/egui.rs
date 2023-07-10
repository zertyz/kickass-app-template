#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use super::lottie_anim_facade::{LOTTIE_ANIMATIONS, LottieAnimation, LottieAnimationFacade};
use super::fractal_clock::{self,FractalClock};
use std::{
    default::Default,
};
use eframe::{
    egui::{self,RichText},
};


#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Egui {
    hello_label:               String,
    //#[serde(skip)]
    hello_value:               f32,
    show_hello_window:         bool,
    show_fractal_clock_window: bool,
    fractal_clock:             FractalClock,
    play_lottie_animation:     bool,
    #[serde(skip)]
    lottie_animations:         Vec<LottieAnimationData>,
}

struct LottieAnimationData {
    selected:       bool,
    animation_name: String,
    animation_data: String,
    animation:      Option<LottieAnimation>,
}

impl Egui {
    pub fn new(label: String, value: f32) -> Self {
        Self {
            hello_label:               label,
            hello_value:               value,
            show_hello_window:         false,
            show_fractal_clock_window: false,
            play_lottie_animation:     true,
            fractal_clock:             FractalClock::default(),
            lottie_animations:         LOTTIE_ANIMATIONS.into_iter()
                .map(|(anim_name, anim_data)| LottieAnimationData {
                    selected: false,
                    animation_name: anim_name.to_string(),
                    animation_data: anim_data.to_string(),
                    animation: None,
                }).collect(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn run_egui_web_app() -> eframe::Result<()> {
        // Redirect `log` message to `console.log` and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();
        
        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
            eframe::WebRunner::new()
                .start(
                    "the_canvas_id", // hardcode it
                    web_options,
                    Box::new(|cc| Box::new(Self::app_creator(cc, "Web Dom", 4.4))),
                )
                .await
                .expect("Running a web eframe");
        });
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_egui_native_app() -> Result<(), Box<dyn std::error::Error>> {
        // Log to stdout (if you run with `RUST_LOG=debug`). -- if you'd ever want it, add to Cargo.toml: tracing-subscriber = "0.3"
        //tracing_subscriber::fmt::init();

        let options = eframe::NativeOptions {
            drag_and_drop_support: false,
            ..Default::default()
        };
        eframe::run_native(
            "kickass-egui-web-app-template",
            options,
            Box::new(|cc| Box::new(Self::app_creator(cc, "Native Dom", 4.4))),
        ).map_err(|err| Box::from(format!("Error running a native eframe: {err}")))
    }

    fn app_creator<IntoString: Into<String>>(cc: &eframe::CreationContext<'_>, default_label: IntoString, default_value: f32) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals {
            dark_mode: true,
            ..Default::default()
        });

        // Load any previous app state or create one from the given parameters -- depends on the `persistence` feature on eframe
        match cc.storage {
            Some(storage) => eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
            None => Self::new(default_label.into(), default_value),
        }
    }
}

impl Default for Egui {
    fn default() -> Self {
        Self::new(String::from("Dom"), 4.4)
    }
}

impl eframe::App for Egui {

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            hello_label: label,
            hello_value: value,
            show_hello_window,
            show_fractal_clock_window,
            ..
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more example, go to https://emilk.github.io/egui

        use chrono::Timelike;
        let time = chrono::Local::now().time();
        let seconds = time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64);

        #[cfg(not(target_arch = "wasm32"))]     // no File->Quit on web pages! -- look at how lottie animations are handled for a better architecture
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.add(egui::Checkbox::new(show_hello_window, "Show 'hello' window"));
            ui.add(egui::Checkbox::new(show_fractal_clock_window, "Show 'fractal clock' window"));

            ui.add(egui::Label::new(RichText::new("Lottie Animations:").size(20.0).underline()));
            for mut animation_data in &mut self.lottie_animations {
                let response = ui.selectable_label(animation_data.selected, &animation_data.animation_name);
                if response.clicked() {
                    if animation_data.selected == false {
                        animation_data.selected = true;
                        animation_data.animation = Some (
                            LottieAnimation::from_data(animation_data.animation_name.to_string(), animation_data.animation_data.to_string())
                        );
                    } else {
                        animation_data.selected = false;
                        animation_data.animation = None;
                    }
                }
                // show the animation window
                if animation_data.selected {
                    egui::Window::new(&animation_data.animation_name).show(ctx, |ui| {
                        let lottie_animation = animation_data.animation.as_mut().unwrap();
                        lottie_animation.show(ui, seconds);
                    });
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        if *show_hello_window {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }

        if *show_fractal_clock_window {
            egui::Window::new("Fractal Clock").show(ctx, |ui| {
                self.fractal_clock.show(ui, Some(seconds));
            });
        }
    }

    /// Called by the frame work to save state before shutdown.
    fn save<'a, 'b>(&'a mut self, storage: &'b mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
