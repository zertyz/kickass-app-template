
use eframe::egui::Ui;

pub trait LottieAnimationFacade {
    /// instantiates (loads) an animation so it may be "played"
    fn from_data(animation_name: String, animation_data: String) -> Self;

    /// "plays" an animation, if it is time to do so
    fn show(&mut self, ui: &mut Ui, seconds: f64);
}