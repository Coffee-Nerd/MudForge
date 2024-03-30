use crate::app::ansi_color::COLOR_MAP;
use egui::{Color32, ScrollArea};
use libmudtelnet::events::TelnetEvents;
use libmudtelnet::Parser; // Adjusted import for TelnetEvents
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Instant;

pub struct TelnetClient {
    stream: Option<TcpStream>,
    pub connection_open: bool,
    received_data: Vec<Vec<(String, Color32)>>,
    parser: Parser,
}

impl TelnetClient {
    pub fn new() -> Self {
        Self {
            stream: None,
            connection_open: false,
            received_data: Vec::new(),
            parser: Parser::new(),
        }
    }
    pub fn append_text(&mut self, text: &str, color: Color32) {
        println!("Appending to Telnet: {}", text); // Debug print
        self.received_data.push(vec![(text.to_string(), color)]);
    }

    pub fn connect(&mut self, ip_address: &str, port: u16) -> Result<(), String> {
        let addr = format!("{}:{}", ip_address, port);
        let socket_addr = addr
            .to_socket_addrs()
            .map_err(|e| format!("Invalid address: {}", e))?
            .next()
            .ok_or("Invalid address")?;

        let stream =
            TcpStream::connect(socket_addr).map_err(|e| format!("Connection failed: {}", e))?;
        stream
            .set_nonblocking(true)
            .map_err(|e| format!("Failed to set non-blocking mode: {}", e))?;
        self.stream = Some(stream);
        self.connection_open = true;
        println!("Connected to {}:{}", ip_address, port);
        Ok(())
    }

    pub fn read_nonblocking(&mut self) -> Option<Vec<(String, Color32)>> {
        if let Some(ref mut stream) = self.stream {
            let mut buffer = [0; 8194]; // Increase buffer size is original, but this fixes parsing issues...//let mut buffer = [0; 1024];
            match stream.read(&mut buffer) {
                Ok(size) if size > 0 => {
                    let events = self.parser.receive(&buffer[..size]);
                    let parsed_text = self.handle_telnet_events(events);
                    // Print raw data in hexadecimal
                    println!("Raw incoming data (hex): {:02X?}", buffer);

                    // Append the parsed text-color pairs to received_data
                    self.received_data.extend(parsed_text.clone());

                    // Debug message for received text
                    println!("Received text: {:?}", parsed_text);
                    Some(parsed_text.into_iter().flatten().collect())
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => None, // Non-blocking read would block
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<(), String> {
        if let Some(ref mut stream) = self.stream {
            match stream.write_all(buffer) {
                Ok(_) => Ok(()),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(()), // Non-blocking write would block
                Err(e) => Err(format!("Write error: {}", e)),
            }
        } else {
            Err("Not connected".to_string())
        }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if self.connection_open {
            let start_time = Instant::now();
            egui::Window::new("Telnet Connection")
                .open(&mut self.connection_open)
                .resizable(true)
                //.frame(egui::Frame{fill:egui::Color32::TRANSPARENT, ..Default::default()})
                .show(ctx, |ui| {
                    ScrollArea::vertical()
                        .auto_shrink(false)
                        .stick_to_bottom(true)
                        .show_rows(
                            ui,
                            ui.text_style_height(&egui::TextStyle::Body),
                            self.received_data.len(),
                            |ui, row_range| {
                                for row in row_range {
                                    let line = &self.received_data[row];
                                    let mut job = egui::text::LayoutJob::default();
                                    for (text, color) in line {
                                        job.append(
                                            text,
                                            0.0,
                                            egui::text::TextFormat {
                                                font_id: ui.style().text_styles
                                                    [&egui::TextStyle::Body]
                                                    .clone(),
                                                color: *color,
                                                ..Default::default()
                                            },
                                        );
                                    }
                                    ui.add(egui::Label::new(job));
                                }
                                let elapsed = start_time.elapsed();
                                //  println!("Visible rows rendered in: {:?}", elapsed);
                            },
                        );
                });
        }
    }

    fn handle_telnet_events(&mut self, events: Vec<TelnetEvents>) -> Vec<Vec<(String, Color32)>> {
        let mut parsed_data: Vec<Vec<(String, Color32)>> = Vec::new();

        for event in events {
            match event {
                TelnetEvents::DataReceive(data) => {
                    // Convert Bytes to Vec<u8> before passing to parse_ansi_codes
                    let parsed_text = parse_ansi_codes(data.to_vec());
                    parsed_data.extend(parsed_text);
                }
                TelnetEvents::DataSend(data) => {
                    if let Some(ref mut stream) = self.stream {
                        let _ = stream.write_all(&data);
                    }
                }
                _ => {
                    // Handle other TelnetEvents as needed
                }
            }
        }

        parsed_data
    }
}

impl Default for TelnetClient {
    fn default() -> Self {
        println!("Creating default TelnetClient instance");
        Self::new()
    }
}

enum AnsiState {
    Normal,
    Escaped,
    Parsing(Vec<u8>),
}

pub fn parse_ansi_codes(buffer: Vec<u8>) -> Vec<Vec<(String, Color32)>> {
    let mut results: Vec<Vec<(String, Color32)>> = Vec::new();
    let mut current_line: Vec<(String, Color32)> = Vec::new();
    let mut current_text = String::new();
    let mut current_color = Color32::WHITE; // Default color
    let mut state = AnsiState::Normal;

    for byte in buffer {
        match state {
            AnsiState::Normal => {
                if byte == 0x1B {
                    // ESC character
                    state = AnsiState::Escaped;
                    if !current_text.is_empty() {
                        current_line.push((current_text.clone(), current_color));
                        current_text.clear();
                    }
                } else {
                    current_text.push(byte as char);
                }
            }
            AnsiState::Escaped => {
                if byte == b'[' {
                    // CSI character
                    state = AnsiState::Parsing(Vec::new());
                } else {
                    state = AnsiState::Normal;
                }
            }
            AnsiState::Parsing(ref mut buf) => {
                if byte == b'm' {
                    // End of ANSI code
                    let code = String::from_utf8_lossy(buf).to_string();
                    if let Some(new_color) = COLOR_MAP.get(code.as_str()) {
                        current_color = *new_color;
                    }
                    buf.clear();
                    state = AnsiState::Normal;
                } else if byte.is_ascii_digit() || byte == b';' {
                    buf.push(byte);
                } else {
                    state = AnsiState::Normal; // Unexpected byte, abort ANSI sequence.
                }
            }
        }
    }

    if !current_text.is_empty() {
        current_line.push((current_text, current_color));
    }

    if !current_line.is_empty() {
        results.push(current_line);
    }

    results
}
