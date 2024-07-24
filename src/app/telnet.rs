use crate::app::ansi_color::COLOR_MAP;
use egui::{Color32, ScrollArea};
use libmudtelnet::events::TelnetEvents;
use libmudtelnet::Parser;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
pub struct TelnetClient {
    stream: Option<TcpStream>,
    pub connection_open: bool,
    pub received_data: Vec<Vec<(String, Color32)>>,
    parser: Parser,
    incomplete_sequence: Vec<u8>, // Buffer for incomplete ANSI sequences
    write_queue: VecDeque<Vec<u8>>, // Queue for outgoing data
}

impl TelnetClient {
    pub fn new() -> Self {
        Self {
            stream: None,
            connection_open: false,
            received_data: Vec::new(),
            parser: Parser::new(),
            incomplete_sequence: Vec::new(),
            write_queue: VecDeque::new(),
        }
    }

    pub fn append_text(&mut self, text: &str, color: Color32) {
        self.received_data.push(vec![(text.to_string(), color)]);
    }

    pub fn append_text_with_colours(
        &mut self,
        text: &str,
        text_colour: Color32,
        back_colour: Color32,
    ) {
        self.received_data
            .push(vec![(text.to_string(), text_colour)]);
    }

    pub fn append_ansi_text(&mut self, text: &str) {
        let parsed_segments = parse_ansi_codes(text.as_bytes().to_vec());
        self.received_data.extend(parsed_segments);
    }

    pub fn connect(&mut self, ip_address: &str, port: &str) -> Result<(), String> {
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
        Ok(())
    }

    pub fn read_nonblocking(&mut self) -> Option<Vec<(String, Color32)>> {
        if let Some(ref mut stream) = self.stream {
            let mut buffer = [0; 8192];
            match stream.read(&mut buffer) {
                Ok(size) if size > 0 => {
                    let mut data = self.incomplete_sequence.clone();
                    data.extend_from_slice(&buffer[..size]);

                    let events = self.parser.receive(&data);
                    let parsed_text = self.handle_telnet_events(events);

                    self.received_data.extend(parsed_text.clone());

                    self.incomplete_sequence = data;

                    Some(parsed_text.into_iter().flatten().collect())
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<(), String> {
        self.write_queue.push_back(buffer.to_vec());
        self.flush_write_queue()
    }

    fn flush_write_queue(&mut self) -> Result<(), String> {
        if let Some(ref mut stream) = self.stream {
            while let Some(data) = self.write_queue.pop_front() {
                match stream.write_all(&data) {
                    Ok(_) => continue,
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        self.write_queue.push_front(data);
                        break;
                    }
                    Err(e) => return Err(format!("Write error: {}", e)),
                }
            }
        }
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if self.connection_open {
            egui::Window::new("Telnet Connection")
                .open(&mut self.connection_open)
                .resizable(true)
                .show(ctx, |ui| {
                    let scroll_area = ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .stick_to_bottom(true);

                    scroll_area.show(ui, |ui| {
                        for (row_index, line) in self.received_data.iter().enumerate() {
                            let mut job = egui::text::LayoutJob::default();
                            for (text, color) in line {
                                job.append(
                                    text,
                                    0.0,
                                    egui::text::TextFormat {
                                        font_id: ui.style().text_styles[&egui::TextStyle::Body]
                                            .clone(),
                                        color: *color,
                                        ..Default::default()
                                    },
                                );
                            }
                            ui.add(egui::Label::new(job));
                        }
                    });
                });
        }
    }

    fn handle_telnet_events(&mut self, events: Vec<TelnetEvents>) -> Vec<Vec<(String, Color32)>> {
        let mut parsed_data: Vec<Vec<(String, Color32)>> = Vec::new();

        for event in events {
            match event {
                TelnetEvents::DataReceive(data) => {
                    let parsed_text = parse_ansi_codes(data.to_vec());
                    parsed_data.extend(parsed_text);
                }
                TelnetEvents::DataSend(data) => {
                    if let Some(ref mut stream) = self.stream {
                        let _ = stream.write_all(&data);
                    }
                }
                _ => {}
            }
        }

        parsed_data
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

pub fn parse_ansi_codes(buffer: Vec<u8>) -> Vec<Vec<(String, Color32)>> {
    let mut results: Vec<Vec<(String, Color32)>> = Vec::new();
    let mut current_line: Vec<(String, Color32)> = Vec::new();
    let mut current_text = String::new();
    let mut current_color = Color32::WHITE;
    let mut state = AnsiState::Normal;

    for byte in buffer {
        match state {
            AnsiState::Normal => {
                if byte == 0x1B {
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
                    state = AnsiState::Parsing(Vec::new());
                } else {
                    state = AnsiState::Normal;
                }
            }
            AnsiState::Parsing(ref mut buf) => {
                if byte == b'm' {
                    let code = String::from_utf8_lossy(buf).to_string();
                    if let Some(new_color) = COLOR_MAP.get(code.as_str()) {
                        current_color = *new_color;
                    }
                    buf.clear();
                    state = AnsiState::Normal;
                } else if byte.is_ascii_digit() || byte == b';' {
                    buf.push(byte);
                } else {
                    state = AnsiState::Normal;
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
