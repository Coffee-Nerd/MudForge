use egui::Color32;

pub struct TelnetState {
    current_foreground: egui::Color32,
    current_background: egui::Color32,
    bold: bool,
    italic: bool,
    underline: bool,
}

impl Default for TelnetState {
    fn default() -> Self {
        Self { 
            current_foreground: egui::Color32::WHITE, // Default to white
            current_background: egui::Color32::BLACK, // Default to black
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

pub fn parse_ansi_codes(data: Vec<u8>, state: &mut TelnetState) -> egui::RichText {
    let mut colored_text = egui::RichText::default();
    let mut current_color = Color32::WHITE;
    let mut i = 0;

    while i < data.len() {
        if data[i] == 27 && data[i + 1] == b'[' {
            let mut code_end = i;
            while code_end < data.len() && data[code_end] != b'm' {
                code_end += 1;
            }
            let code = &data[i + 2..code_end];
            i = code_end + 1;

            if code.starts_with(b"30") {
                current_color = Color32::BLACK;
            } else if code.starts_with(b"31") {
                current_color = Color32::RED;
            } else if code.starts_with(b"32") {
                current_color = Color32::GREEN;
            } else if code.starts_with(b"33") {
                current_color = Color32::YELLOW;
            } else if code.starts_with(b"34") {
                current_color = Color32::BLUE;
            } else if code.starts_with(b"35") {
                current_color = Color32::from_rgb(255, 0, 255); // MAGENTA
            } else if code.starts_with(b"36") {
                current_color = Color32::from_rgb(0, 255, 255); // CYAN
            } else if code.starts_with(b"37") {
                current_color = Color32::LIGHT_GRAY;
            } else if code == b"0" {
                current_color = Color32::WHITE;
            }
        } else {
            let text_segment = if data[i] == 13 || data[i] == 10 {
                // Handle newline characters (\r and \n)
                egui::RichText::new("\n").color(current_color)
            } else {
                egui::RichText::new(std::str::from_utf8(&data[i..i + 1]).unwrap())
                    .color(current_color)
            };
            colored_text = egui::RichText::new(format!("{}{}", colored_text.text(), text_segment.text()));
            i += 1;
        }
    }

    colored_text
}