use egui::Window;
use std::cell::RefCell; 
mod telnet;
mod miniwindow;
mod parse_colors;
use miniwindow::WindowResizeTest;


#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct TemplateApp {
    label: String,
    value: f32,
    window_resize_test: WindowResizeTest,
    #[serde(skip)]
    telnet_client: telnet::TelnetClient,
    show_connection_prompt: RefCell<bool>, // Using RefCell
    ip_address: String,
    port: u16,
    command: String,
}


impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
            window_resize_test: WindowResizeTest::new(),
            telnet_client: telnet::TelnetClient::new(),
            show_connection_prompt: RefCell::new(false), // Initialize
            ip_address: "127.0.0.1".to_owned(),
            port: 23,
            command: String::new(),
        }
    }
}

impl eframe::App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let menus: &[(&str, Vec<(&str, Box<dyn Fn(&mut Self, &egui::Context)>)>)] = &[
            ("File", vec![
                ("Quit", Box::new(|_, ctx| ctx.send_viewport_cmd(egui::ViewportCommand::Close))),  
            ]),
            ("Connection", vec![
                ("New", Box::new(|s, _| { s.show_connection_prompt.replace(true); })), // Updated
            ]),
        ];

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                for &(menu_name, ref submenus) in menus {
                    ui.menu_button(menu_name, |ui| {
                        for &(submenu_name, ref action) in submenus {
                            if ui.button(submenu_name).clicked() {
                                action(self, ctx);
                            }
                        }
                    });
                }
                ui.add_space(16.0);
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
                ui.separator();
        
                // Make the CentralPanel fill the entire width of the window
                ui.set_max_width(ui.available_size().x);
        
                // Add a text input field for the command
                let mut command = String::new();
                ui.horizontal(|ui| {
                    ui.code_editor(&mut self.command);
                    if ui.button("Send").clicked() | ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.command.is_empty() {
                            // Append a newline character if required by the Telnet server
                            self.command.push('\n');
                            if let Err(e) = self.telnet_client.write(self.command.as_bytes()) {
                                eprintln!("Failed to send command: {}", e);
                            }
                            self.command.clear();
                        } else {
                            println!("Command is empty"); // Debug print
                        }
                    }
                });
            });
        });

        let mut open = *self.show_connection_prompt.borrow(); 
        let mut close_window = false; 

        if open {
            egui::Window::new("Connect to Telnet Server")
                .open(&mut open)
                .vscroll(true)
                .resizable(true)
                .default_height(300.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                    ui.label("Ip Address:    ");
                    ui.text_edit_singleline(&mut self.ip_address);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Port number:");
                        ui.text_edit_singleline(&mut self.port.to_string());
                        });
                    if ui.button("Connect").clicked() {
                        if let Err(e) = self.telnet_client.connect(&self.ip_address, self.port) {
                            eprintln!("Connection error: {}", e);
                        }
                        close_window = true; 
                    }
                });

            if close_window {
                *self.show_connection_prompt.borrow_mut() = false;
            }
        }

        if self.telnet_client.is_connected() {
            if let Some(data) = self.telnet_client.read_nonblocking() {
            //    println!("Received data: {}", data.text());
            }
        }

        self.window_resize_test.show(ctx);
        self.telnet_client.show(ctx);
    }
}