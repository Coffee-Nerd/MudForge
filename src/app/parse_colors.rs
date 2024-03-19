use egui::{Color32, RichText, TextStyle, Ui};

// This structure represents a segment of styled text.
pub struct StyledTextSegment {
   pub text: String,
   pub  color: Color32,
}

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

// Parses the given byte slice into styled text segments.
pub fn parse_ansi_codes_to_styled_text(buffer: &[u8]) -> Vec<StyledTextSegment> {
    let mut segments: Vec<StyledTextSegment> = Vec::new();
    let mut current_color = Color32::WHITE; // Default color
    let mut current_text = String::new();

    let mut chars = buffer.iter().peekable();
    while let Some(&byte) = chars.next() {
        match byte {
            // Beginning of the ANSI sequence
            b'\x1b' => {
                // Push the current text as a segment before resetting it.
                if !current_text.is_empty() {
                    segments.push(StyledTextSegment {
                        text: current_text.clone(),
                        color: current_color,
                    });
                    current_text.clear();
                }

                // Read until 'm' which signifies the end of the color code sequence.
                while chars.peek() != Some(&&&b'm') {
                    chars.next();
                }
                // Consume the 'm' byte.
                chars.next();

                // This is where you would parse the actual numbers between '\x1b[' and 'm'
                // to set the current_color to the appropriate `Color32` value.
                // For simplicity, let's assume all text following is red.
                current_color = Color32::RED;
            }
            // New line or carriage return
            b'\r' | b'\n' => {
                if byte == b'\n' {
                    current_text.push('\n');
                }
            }
            // Regular character
            _ => {
                let ch = byte as char;
                current_text.push(ch);
            }
        }
    }

    // Push the last text segment if any.
    if !current_text.is_empty() {
        segments.push(StyledTextSegment {
            text: current_text,
            color: current_color,
        });
    }

    segments
}

// Function to display the styled text segments in egui.
pub fn display_styled_text(ui: &mut Ui, segments: Vec<StyledTextSegment>) {
    for segment in segments {
        ui.label(RichText::new(&segment.text).color(segment.color));
    }
}
