use std::net::ToSocketAddrs;
use telnet::{Telnet, Event};

pub struct TelnetClient {
    client: Option<Telnet>,
    pub connection_open: bool,
}

impl TelnetClient {
    pub fn new() -> Self {
        Self {
            client: None,
            connection_open: false,
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
        self.connection_open = true;
        Ok(())
    }

    pub fn read_nonblocking(&mut self) -> Option<Vec<u8>> {
        if let Some(ref mut client) = self.client {
            match client.read_nonblocking().expect("Read error") {
                Event::Data(buffer) => Some(buffer.to_vec()),
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
            egui::Window::new("Telnet Connection").show(ctx, |ui| {
                ui.label("Connected to Telnet server.");
            });
        }
    }
}

impl Default for TelnetClient {
    fn default() -> Self {
        Self::new()
    }
}
