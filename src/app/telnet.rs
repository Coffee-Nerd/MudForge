use std::net::ToSocketAddrs;
use telnet::{Telnet, Event, TelnetOption, Action}; 
use crate::app::colors::parse_ansi_codes;
use crate::app::colors::TelnetState;


pub struct TelnetClient {
    client: Option<Telnet>,
    pub connection_open: bool,
    received_data: egui::RichText,  // Add a field to store received data
    pub received_text: Vec<(String, String)>,
    telnet_state: TelnetState
}

impl TelnetClient {
    pub fn new() -> Self {
        Self {
            client: None,
            connection_open: false,
            received_data: egui::RichText::default(),  // Initialize the received data string
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

        self.connection_open = true;
        Ok(())
    }
    
    
    
    

    pub fn read_nonblocking(&mut self) -> Option<egui::RichText> {
        if let Some(ref mut client) = self.client {
            match client.read_nonblocking().expect("Read error") {
                Event::Data(buffer) => {
                    // Print raw data in hexadecimal
                    println!("Raw incoming data (hex): {:02X?}", buffer);
                    println!("SEPARATION LINE"); // Keep this if you find it helpful
    
                    let parsed_text = parse_ansi_codes(buffer.to_vec(), &mut self.telnet_state);
    
                    // Print the parsed text
                    println!("Parsed text: {}", parsed_text.text());
    

// Concatenate using format!()
self.received_data = egui::RichText::new(format!("{}{}", self.received_data.text(), parsed_text.text()));
    
                    // Print the updated received_data
                    println!("Updated received_data: {}", self.received_data.text());
    
                    Some(self.received_data.clone())
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
                    ui.add(egui::Label::new(self.received_data.clone())); // Display the colored text directly
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