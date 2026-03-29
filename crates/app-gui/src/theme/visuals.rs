use egui::{epaint::Shadow, Color32, Context, Stroke, Visuals};
use super::colors::*;

pub fn setup_visuals(ctx: &Context) {
    let mut visuals = Visuals::dark();

    // Base background colors
    visuals.window_fill = SURFACE_CONTAINER_HIGHEST;
    visuals.panel_fill = SURFACE;

    // Text colors
    visuals.override_text_color = Some(ON_SURFACE);
    
    // UI Elements
    visuals.widgets.noninteractive.bg_fill = SURFACE_CONTAINER;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, OUTLINE_VARIANT.linear_multiply(0.15));
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, ON_SURFACE);

    visuals.widgets.inactive.bg_fill = SURFACE_CONTAINER_LOW;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, OUTLINE_VARIANT.linear_multiply(0.15));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, ON_SURFACE_VARIANT);

    visuals.widgets.hovered.bg_fill = SURFACE_CONTAINER_HIGH;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, SECONDARY);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, ON_SURFACE);

    visuals.widgets.active.bg_fill = SURFACE_CONTAINER_HIGHEST;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, PRIMARY);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, ON_PRIMARY);

    // Selections
    visuals.selection.bg_fill = SECONDARY.linear_multiply(0.3);
    visuals.selection.stroke = Stroke::new(1.0, SECONDARY);

    // Shapes
    // Remove window stroke completely if possible, or use faint OUTLINE_VARIANT
    visuals.window_stroke = Stroke::new(1.0, OUTLINE_VARIANT.linear_multiply(0.3));

    visuals.popup_shadow = Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(100),
    };

    ctx.set_visuals(visuals);
}
