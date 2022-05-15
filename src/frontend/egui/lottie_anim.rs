//! as of 2022-05-12, egui is still zero-copy unfriendly, so we circumvent this performance hit by
//! "pre-loading" all frames of the animation into bitmap buffers (textures), at the cost of increased RAM usage
//! (this is done, here, on the UI thread...)


use eframe::egui::{ColorImage, TextureHandle, Ui};
pub use rlottie::{Animation,Surface};
use rgb::{alt::BGRA8};


pub struct LottieAnimation {
    painting_width:  usize,
    painting_height: usize,
    texture_cache:   Vec<Option<TextureHandle>>,
    repaint_counter: usize,
    lottie_player:   Animation,
    rlottie_surface: Surface,
    rgba_buffer:     Vec<u8>,
}

impl LottieAnimation {

    pub fn from_data(animation_name: String, animation_data: String) -> Self {
        let lottie_player = Animation::from_data(
            animation_data.to_string(),
            animation_name,
            "from data").expect("Failed to interpret data for lottie animation");
        let width = lottie_player.size().width;
        let height = lottie_player.size().height;
        let mut rgba_buffer = Vec::<u8>::with_capacity(4*width*height);
        for _ in 0..4*width*height {
            rgba_buffer.push(64);   // fill the bitmap with a lower gray
        }
        Self {
            painting_width: lottie_player.size().width,
            painting_height: lottie_player.size().height,
            texture_cache: (0..lottie_player.totalframe()).into_iter().map(|_| None).collect(),
            repaint_counter: 0,
            lottie_player,
            rlottie_surface: Surface::new(rlottie::Size {width, height} ),
            rgba_buffer,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, _seconds: f64) {
        let max_size = ui.available_size();
        let frame_number = self.repaint_counter % self.lottie_player.totalframe();
        self.repaint_counter += 1;

        // when the paint area is resized, we invalidate our existing textures
        if self.painting_width != max_size.x as usize || self.painting_height != max_size.y as usize {
            let width = max_size.x as usize;
            let height = max_size.y as usize;
            self.painting_width = width;
            self.painting_height = height;
            self.texture_cache.iter_mut()
                .for_each(|entry| *entry = None);
            // pre-allocate the egui & rlottie buffers
            let bytes_len = 4*self.painting_width*self.painting_height;
            let mut rgba_buffer = Vec::<u8>::with_capacity(bytes_len);
            for _ in 0..bytes_len {
                rgba_buffer.push(196);
            }
            self.rgba_buffer = rgba_buffer;
            self.rlottie_surface = Surface::new(rlottie::Size {width, height} );

        }

        // get the texture (frame) from the cache or build it
        let texture = self.texture_cache.get_mut(frame_number)
            .expect("BUG: not all frame slots have been reserved when Self was created")
            .get_or_insert_with(|| {
                self.lottie_player.render(frame_number, &mut self.rlottie_surface);
                rlottie_bgra_to_u8_rgba(&self.rlottie_surface.data(), &mut self.rgba_buffer);
                let image = ColorImage::from_rgba_unmultiplied([self.painting_width, self.painting_height], &self.rgba_buffer);
                ui.ctx().load_texture(format!("Lottie Animation frame #{}", frame_number), image)
        });

        // paint the texture for this frame and request a repaint for the next one
        ui.image(texture.id(), max_size);
        ui.ctx().request_repaint();
    }
}

/// converts rlottie BGRA pixels into ARGB for egui's texture bitmap
fn rlottie_bgra_to_u8_rgba(rlottie_bgra: &[BGRA8], u8_rgba: &mut [u8]) {
    u8_rgba.chunks_exact_mut(4)
        .zip(rlottie_bgra)
        .for_each(|(rgba, bgra)| {
            rgba[0] = bgra.r;
            rgba[1] = bgra.g;
            rgba[2] = bgra.b;
            rgba[3] = bgra.a;
        });
}