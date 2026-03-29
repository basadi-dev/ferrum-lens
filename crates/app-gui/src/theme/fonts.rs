use egui::FontDefinitions;

pub fn setup_custom_fonts(ctx: &egui::Context) {
    let fonts = FontDefinitions::default();

    // In a real app we'd load Manrope/Inter/JetBrains Mono from bytes
    // For now we map default proportional to our logic
    
    // We can just rely on default setup for the basic template,
    // but here is where we would inject:
    // fonts.font_data.insert("Inter".to_owned(), FontData::from_static(include_bytes!("../../assets/Inter-Regular.ttf")));
    
    ctx.set_fonts(fonts);
}
