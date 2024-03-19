use std::net::ToSocketAddrs;
use telnet::{Telnet, Event, TelnetOption, Action}; 
use crate::app::parse_colors::TelnetState;
use egui::{Color32, Context, FontId, TextFormat, TextEdit, Ui, TextStyle};
use std::sync::Arc;
use crate::app::parse_colors;

pub struct TelnetClient {
    client: Option<Telnet>,
    pub connection_open: bool,
    received_data: Vec<(String, Color32)>,
    pub received_text: Vec<(String, String)>,
    telnet_state: TelnetState
}

impl TelnetClient {
    pub fn new() -> Self {
        Self {
            client: None,
            connection_open: false,
            received_data: Vec::new(),
            received_text: Vec::new(),
            telnet_state: TelnetState::default()
        }
    }

    pub fn connect(&mut self, ip_address: &str, port: u16) -> Result<(), String> {
        let addr = format!("{}:{}", ip_address, port);
        let socket_addr = addr
            .to_socket_addrs()
            .map_err(|e| format!("Invalid address: {}", e))?
            .next()
            .ok_or("Invalid address")?;

        self.client = Some(
            Telnet::connect(socket_addr, 256)
                .map_err(|e| format!("Connection failed: {}", e))?,
        );

        // Enable the "Terminal Type" option
        if let Some(ref mut client) = self.client {
            // Create a vector to hold the option settings
            let mut options = Vec::new();
            options.push((TelnetOption::TTYPE, Some(b"xterm-256color")));
        
 // Negotiate each option individually
 for (option, value) in options {
    // Determine the appropriate action (Do, Will, etc.)
    let action = Action::Will;

    client.negotiate(&action, option).expect("Failed to negotiate options");
}
        }
        // Debug message for successful connection
        println!("Connected to {}:{}", ip_address, port);
        self.connection_open = true;
        Ok(())
    }
    
    
    
    

    pub fn read_nonblocking(&mut self) -> Option<Vec<(String, Color32)>> {
        if let Some(ref mut client) = self.client {
            match client.read_nonblocking().expect("Read error") {
                Event::Data(buffer) => {
                    // Print raw data in hexadecimal
                    println!("Raw incoming data (hex): {:02X?}", buffer);
    
                    let parsed_text = parse_ansi_codes(buffer.to_vec(), &mut self.telnet_state);
    
                    // Append the parsed text-color pairs to received_data
                    self.received_data.extend(parsed_text.clone());
    
                    // Debug message for received text
                    println!("Received text: {:?}", parsed_text);
    
                    Some(parsed_text)
                }
                _ => None,
            }
        } else {
            None
        }
    }
    

    pub fn write(&mut self, buffer: &[u8]) -> Result<(), String> {
        if let Some(ref mut client) = self.client {
            client.write(buffer).map_err(|e| format!("Write error: {}", e))?;
            Ok(())
        } else {
            Err("Not connected".to_string())
        }
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if self.connection_open {
            egui::Window::new("Telnet Connection")
                .vscroll(true)
                .resizable(true)
                .frame(egui::Frame::none().fill(egui::Color32::BLACK))
                .show(ctx, |ui| {
                    // Create a new LayoutJob
                    let mut job = egui::text::LayoutJob::default();
    
                    // Define the default text style 
                    let font_id = ui.style().text_styles[&egui::TextStyle::Body].clone();
    
                    for (text, color) in &self.received_data {
                        // Handle newlines within text
                        for line in text.split('\n') {
                            if !line.is_empty() {
                                // Add each line of text along with its color to the LayoutJob
                                job.append(
                                    line,
                                    0.0, // Words spacing
                                    egui::text::TextFormat {
                                        font_id: font_id.clone(),
                                        color: *color,
                                        ..Default::default()
                                    },
                                );
                            }
                            // Insert a newline in the LayoutJob if there was one in the original text
                            if text.contains('\n') {
                                job.append("\n", 0.0, egui::text::TextFormat::default());
                            }
                        }
                    }
    
                    // Add the LayoutJob to the Ui
                    ui.label(job);
    
                    // Auto-scroll to the bottom
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                });
        }
    }
    
}    

impl Default for TelnetClient {
    fn default() -> Self {
        Self::new()
    }
}


fn default_highlighter(ui: &Ui, string: &str) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let font_id = ui.style().text_styles[&TextStyle::Body].clone(); // Clone the FontId for the body text style

    for word in string.split_whitespace() {
        let color = match word {
            // Example: colorize "error" words in red
            "error" => Color32::RED,
            _ => Color32::WHITE,
        };
        job.append(word, 0.0, TextFormat::simple(font_id.clone(), color)); // Clone the FontId for each word
    }
    job
}



enum AnsiState {
    Normal,
    Escaped,
    Parsing(Vec<u8>),
}

fn parse_ansi_codes(buffer: Vec<u8>, telnet_state: &mut TelnetState) -> Vec<(String, Color32)> {
    let mut results = Vec::new();
    let mut current_text = String::new();
    let mut current_color = Color32::WHITE; // Default color
    let mut state = AnsiState::Normal;

    for byte in buffer {
        match state {
            AnsiState::Normal => if byte == 0x1B {
                // If there is text accumulated with the current color, push it into results.
                if !current_text.is_empty() {
                    results.push((current_text.clone(), current_color));
                    current_text.clear();
                }
                state = AnsiState::Escaped;
            } else {
                current_text.push(byte as char);
            },
            AnsiState::Escaped => if byte == b'[' {
                state = AnsiState::Parsing(Vec::new());
            } else {
                state = AnsiState::Normal;
            },
            AnsiState::Parsing(ref mut buf) => if byte == b'm' {
                // End of ANSI sequence, determine color.
                current_color = match buf.as_slice() {
                    b"0;32" => Color32::from_rgb(0, 128, 0), // Example color
                    // Add other color cases here
                    _ => Color32::WHITE, // Default to white for unknown codes
                };
                buf.clear();
                state = AnsiState::Normal;
            } else if (byte as char).is_digit(10) || byte == b';' {
                buf.push(byte);
            } else {
                // Unexpected byte, abort ANSI sequence.
                state = AnsiState::Normal;
            },
        }
    }

    // If there's text left after parsing, push it into results.
    if !current_text.is_empty() {
        results.push((current_text, current_color));
    }

    results
}



fn parse_ansi_escape_sequence(sequence: &[u8], telnet_state: &mut TelnetState) -> (String, Color32) {
    let seq_str = String::from_utf8_lossy(sequence);
    let color = match seq_str.as_ref() {
        "0;30m" => Color32::from_rgb(0, 0, 0),      // Black
        "0;31m" => Color32::from_rgb(128, 0, 0),    // Dark Red
        "0;32m" => Color32::from_rgb(0, 128, 0),    // Dark Green
        "0;33m" => Color32::from_rgb(128, 128, 0),  // Dark Yellow
        "0;34m" => Color32::from_rgb(0, 0, 128),    // Dark Blue
        "0;35m" => Color32::from_rgb(128, 0, 128),  // Dark Magenta
        "0;36m" => Color32::from_rgb(0, 128, 128),  // Dark Cyan
        "0;37m" => Color32::from_rgb(192, 192, 192),// Light Gray
        "1;30m" => Color32::from_rgb(128, 128, 128),// Dark Gray
        "1;31m" => Color32::from_rgb(255, 0, 0),    // Red
        "1;32m" => Color32::from_rgb(0, 255, 0),    // Green
        "1;33m" => Color32::from_rgb(255, 255, 0),  // Yellow
        "1;34m" => Color32::from_rgb(0, 0, 255),    // Blue
        "1;35m" => Color32::from_rgb(255, 0, 255),  // Magenta
        "1;36m" => Color32::from_rgb(0, 255, 255),  // Cyan
        "1;37m" => Color32::from_rgb(255, 255, 255),// White
        _ => Color32::from_rgb(255, 255, 255),      // Default to white if unknown
    };
    (seq_str.to_string(), color)
}

