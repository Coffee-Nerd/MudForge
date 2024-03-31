use egui::{Color32, FontData, FontDefinitions, FontFamily, Stroke, Style};

pub fn default_style() -> Style {
    let mut style = Style::default();
    style.visuals.extreme_bg_color = Color32::from_rgb(20, 21, 23); // text box background
    style.visuals.faint_bg_color = Color32::from_rgb(45, 46, 48);
    style.visuals.code_bg_color = Color32::from_rgb(45, 51, 59);
    style.visuals.hyperlink_color = Color32::from_rgb(255, 0, 0);
    style.visuals.window_fill = Color32::from_rgb(0, 0, 0); // menu bg, widget bg
    style.visuals.panel_fill = Color32::from_rgb(18, 18, 20); // entire window bg
    style.visuals.override_text_color = Some(Color32::from_rgb(255, 255, 255));
    style.visuals.button_frame = true;
    style.visuals.collapsing_header_frame = true;
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(35, 39, 46);
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(0., Color32::from_rgb(45, 46, 48));
    style.visuals.widgets.active.fg_stroke = Stroke::new(0., Color32::from_rgb(45, 46, 48));
    style.visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 51, 59);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(45, 51, 59);
    style.visuals.widgets.open.bg_fill = Color32::from_rgb(45, 51, 59);
    style
}

pub fn custom_font() -> FontDefinitions {
    let font_ReFixedysMono = include_bytes!("../data/refixedsys-mono.otf").to_vec();
    let mut font = FontDefinitions::default();
    font.font_data.insert(
        "ReFixedys Mono".to_string(),
        FontData::from_owned(font_ReFixedysMono),
    );
    font.families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "ReFixedys Mono".to_string());
    font.families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "ReFixedys Mono".to_string());
    font
}
