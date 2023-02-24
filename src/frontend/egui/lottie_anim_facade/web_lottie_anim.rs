//! When in web, lottie animations could be used without the need of the rlottie library,
//! as browsers natively supports these animations -- it is not implemented here, 'thought

use eframe::egui::{self, Ui, RichText};


pub struct LottieAnimation {
    animation_name: String,
}

impl super::types::LottieAnimationFacade for LottieAnimation {
    fn from_data(animation_name: String, animation_data: String) -> Self {
        Self {
            animation_name
        }
    }

    fn show(&mut self, ui: &mut Ui, _seconds: f64) {
        ui.add(egui::Label::new(RichText::new(format!("Here I'd show lottie animation '{}' by asking the browser to download that file from the server and showing it here...", self.animation_name)).size(15.0)));
    }
}