use std::net::ToSocketAddrs;
use telnet::{Telnet, Event, TelnetOption, Action}; 
use crate::app::colors::TelnetState;
use egui::{Color32, FontId, TextFormat, TextEdit, Ui, TextStyle};
use std::sync::Arc;


pub struct TelnetClient {
    client: Option<Telnet>,
    pub connection_open: bool,
    received_data: String,  // Add a field to store received data
    pub received_text: Vec<(String, String)>,
    telnet_state: TelnetState
}

impl TelnetClient {
    pub fn new() -> Self {
        Self {
            client: None,
            connection_open: false,
            received_data: String::new(),  // Initialize the received data string
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
            // Create a vector to hold your option settings
            let mut options = Vec::new();
            options.push((TelnetOption::TTYPE, Some(b"xterm-256color")));
        
 // Negotiate each option individually
 for (option, value) in options {
    // Determine the appropriate action (Do, Will, etc.)
    let action = Action::Will; // You might need to adjust this

    client.negotiate(&action, option).expect("Failed to negotiate options");
}
        }
        // Debug message for successful connection
        println!("Connected to {}:{}", ip_address, port);
        self.connection_open = true;
        Ok(())
    }
    
    
    
    

    pub fn read_nonblocking(&mut self) -> Option<String> {
        if let Some(ref mut client) = self.client {
            match client.read_nonblocking().expect("Read error") {
                Event::Data(buffer) => {
                    // Print raw data in hexadecimal
                    println!("Raw incoming data (hex): {:02X?}", buffer);

                    let parsed_text = parse_ansi_codes(buffer.to_vec(), &mut self.telnet_state);

                    // Append the parsed text to received_data
                    self.received_data.push_str(&parsed_text);

                    // Debug message for received text
                    println!("Received text: {}", parsed_text);

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
                    // Use TextEdit for displaying received data with syntax highlighting
                    let mut text_edit = TextEdit::multiline(&mut self.received_data)
                        .text_color(Color32::WHITE);  // Set default text color to white
                    
                    // Custom layouter for syntax highlighting
                    let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
                        let mut layout_job: egui::text::LayoutJob = default_highlighter(ui, string);
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                    };
                    text_edit = text_edit.layouter(&mut layouter);
    
                    ui.add(text_edit);
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



// Replace this with your own function for parsing ANSI codes
fn parse_ansi_codes(buffer: Vec<u8>, telnet_state: &mut TelnetState) -> String {
    // Placeholder implementation
    String::from_utf8_lossy(&buffer).into_owned()
}