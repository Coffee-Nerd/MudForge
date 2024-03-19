use egui::Color32;
use lazy_static::lazy_static;

pub fn generate_xterm_color_map() -> Vec<(&'static str, Color32)> {
    let mut color_map = vec![
        ("0;30", Color32::from_rgb(0, 0, 0)),       // Black
        ("0;31", Color32::from_rgb(128, 0, 0)),     // Dark Red
        ("0;32", Color32::from_rgb(0, 128, 0)),     // Dark Green
        ("0;33", Color32::from_rgb(128, 128, 0)),   // Dark Yellow
        ("0;34", Color32::from_rgb(0, 0, 128)),     // Dark Blue
        ("0;35", Color32::from_rgb(128, 0, 128)),   // Dark Magenta
        ("0;36", Color32::from_rgb(0, 128, 128)),   // Dark Cyan
        ("0;37", Color32::from_rgb(192, 192, 192)), // Light Gray
        ("1;30", Color32::from_rgb(128, 128, 128)), // Dark Gray
        ("1;31", Color32::from_rgb(255, 0, 0)),     // Red
        ("1;32", Color32::from_rgb(0, 255, 0)),     // Green
        ("1;33", Color32::from_rgb(255, 255, 0)),   // Yellow
        ("1;34", Color32::from_rgb(0, 0, 255)),     // Blue
        ("1;35", Color32::from_rgb(255, 0, 255)),   // Magenta
        ("1;36", Color32::from_rgb(0, 255, 255)),   // Cyan
        ("1;37", Color32::from_rgb(255, 255, 255)), // White
    ];

    // Generate xterm colors
    for i in 0..256 {
        let r = if i >= 16 && i < 232 {
            (((i - 16) / 36) * 51) as u8 // 0, 51, 102, ...
        } else if i >= 232 {
            (((i - 232) * 255) / 23) as u8    // Linear grayscale
        } else {
            0
        };

        let g = if i >= 16 && i < 232 {
            (((i - 16) % 36 / 6) * 51) as u8
        } else if i >= 232 { 
            (((i - 232) * 255) / 23) as u8    
        } else {
            0
        };

        let b = if i >= 16 && i < 232 {
            (((i - 16) % 6) * 51) as u8
        } else if i >= 232 { 
            (((i - 232) * 255) / 23) as u8     
        } else {
            0
        };

        let color_code = format!("38;5;{}", i);
        let color = Color32::from_rgb(r as u8, g as u8, b as u8);
        color_map.push((Box::leak(color_code.into_boxed_str()), color));
    }

    color_map
}

lazy_static! {
    pub static ref COLOR_MAP: Vec<(&'static str, Color32)> = generate_xterm_color_map();
}