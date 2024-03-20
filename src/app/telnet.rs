use std::net::ToSocketAddrs;
use telnet::{Telnet, Event, TelnetOption, Action}; 
use egui::{Color32, Context, FontId, TextFormat, TextEdit, Ui, TextStyle};
use crate::app::ansi_color::{COLOR_MAP};



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
    
                    let parsed_text = parse_ansi_codes(buffer.to_vec());
    
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
                    ui.allocate_space(ui.available_size());
                });
        }
    }
    
}    

impl Default for TelnetClient {
    fn default() -> Self {
        Self::new()
    }
}

enum AnsiState {
    Normal,
    Escaped,
    Parsing(Vec<u8>),
}

pub fn parse_ansi_codes(buffer: Vec<u8>) -> Vec<(String, Color32)> {
    let mut results = Vec::new();
    let mut current_text = String::new();
    let mut current_color = Color32::WHITE; // Default color
    let mut state = AnsiState::Normal;
    let mut ansi_buffer: Vec<u8> = Vec::new();

    for byte in buffer {
        match state {
            AnsiState::Normal => {
                if byte == 0x1B { // ESC character
                    state = AnsiState::Escaped;
                    if !current_text.is_empty() {
                        results.push((current_text.clone(), current_color));
                        current_text.clear();
                    }
                } else {
                    current_text.push(byte as char);
                }
            },
            AnsiState::Escaped => {
                if byte == b'[' { // CSI character
                    ansi_buffer.clear();
                    state = AnsiState::Parsing(Vec::new());
                } else {
                    state = AnsiState::Normal;
                }
            },
            AnsiState::Parsing(ref mut buf) => {
                if byte == b'm' { // End of ANSI code
                    let code = String::from_utf8_lossy(buf).to_string();
                    if let Some(new_color) = COLOR_MAP.iter()
                        .find(|&&(ansi_code, _)| ansi_code == code)
                        .map(|&(_, color)| color) {
                            current_color = new_color;
                    }
                    buf.clear();
                    state = AnsiState::Normal;
                } else if byte.is_ascii_digit() || byte == b';' {
                    buf.push(byte);
                } else {
                    state = AnsiState::Normal; // Unexpected byte, abort ANSI sequence.
                }
            },
        }
    }

    if !current_text.is_empty() {
        results.push((current_text, current_color));
    }

    results
}