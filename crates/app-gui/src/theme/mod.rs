pub mod colors;
pub mod fonts;
pub mod visuals;

use egui::Context;

pub fn setup_theme(ctx: &Context) {
    fonts::setup_custom_fonts(ctx);
    visuals::setup_visuals(ctx);
}
