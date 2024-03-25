use egui::Window;
use std::cell::RefCell;
mod ansi_color;
mod miniwindow;
mod styles;
mod telnet;
use egui::FontFamily;
use miniwindow::WindowResizeTest;
use std::collections::VecDeque;
use std::time::Instant;
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
    command_history: Vec<String>,
    current_history_index: usize,
    fps: f64,
    #[serde(skip)]
    last_frame_time: Option<Instant>,
    #[serde(skip)]
    last_frame_update: Option<Instant>,
    #[serde(skip)]
    frame_durations: VecDeque<f64>, // Use VecDeque for efficient push/pop operations
    #[serde(skip)]
    last_update_time: Option<Instant>,
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set the custom style
        let style = styles::default_style(); // Use the default style function
        cc.egui_ctx.set_style(style);

        // Set the custom font
        let font = styles::custom_font(); // Use the custom font function
        cc.egui_ctx.set_fonts(font);

        // Initialize the rest of the application
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
            command_history: Vec::new(),
            current_history_index: 0,
            fps: 0.0,
            last_frame_time: None,
            last_frame_update: None,
            frame_durations: VecDeque::with_capacity(10),
            last_update_time: None,
        }
    }
}

impl eframe::App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let menus: &[(&str, Vec<(&str, Box<dyn Fn(&mut Self, &egui::Context)>)>)] = &[
            (
                "File",
                vec![(
                    "Quit",
                    Box::new(|_, ctx| ctx.send_viewport_cmd(egui::ViewportCommand::Close)),
                )],
            ),
            (
                "Connection",
                vec![
                    (
                        "New",
                        Box::new(|s, _| {
                            s.show_connection_prompt.replace(true);
                        }),
                    ), // Updated
                ],
            ),
        ];

        let now = Instant::now();
        if let Some(last_frame_time) = self.last_frame_time {
            let elapsed = now.duration_since(last_frame_time).as_secs_f64();
            self.frame_durations.push_front(elapsed);
            if self.frame_durations.len() > 10 {
                self.frame_durations.pop_back();
            }
        }
        self.last_frame_time = Some(now);

        if let Some(last_frame_update) = self.last_frame_update {
            if now.duration_since(last_frame_update).as_secs() >= 1 {
                // Update average FPS calculation
                let total_duration: f64 = self.frame_durations.iter().sum();
                self.fps = 10.0 / total_duration; // Average over the last 10 seconds

                // Update last update time
                self.last_frame_update = Some(now);
            }
        } else {
            // First update, initialize last update time
            self.last_frame_update = Some(now);
        }

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
                ui.horizontal(|ui| {
                    egui::warn_if_debug_build(ui);
                    ui.label(format!("FPS: {:.0}", self.fps));
                });
                // Make the CentralPanel fill the entire width of the window
                ui.set_max_width(ui.available_size().x);

                // Add a text input field for the command
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    // Calculate the width of the input box to fill the available width
                    let input_box_width = ui.available_size().x - 100.0; // Adjust the subtraction as needed for your layout

                    // Use add_sized to set the size of the TextEdit
                    // Inside the update method, where the command input field is handled
                    let response = ui.add_sized(
                        [
                            input_box_width,
                            ui.text_style_height(&egui::TextStyle::Body),
                        ],
                        |ui: &mut egui::Ui| ui.text_edit_singleline(&mut self.command),
                    );

                    // Check for key presses
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        if self.current_history_index > 0 {
                            self.current_history_index -= 1;
                            self.command = self.command_history[self.current_history_index]
                                .trim()
                                .to_string();
                        } else {
                            // There is no command history, so do nothing
                        }
                    } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        if self.current_history_index < self.command_history.len() - 1 {
                            self.current_history_index += 1;
                            self.command = self.command_history[self.current_history_index]
                                .trim()
                                .to_string();
                        } else {
                            self.command.clear();
                            self.current_history_index = self.command_history.len();
                        }
                    }

                    // Add the command to the history when it's submitted
                    if ui.button("Send").clicked()
                        || response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        if !self.command.is_empty() {
                            // Append a newline character if required by the Telnet server
                            self.command.push('\n');
                            if let Err(e) = self.telnet_client.write(self.command.as_bytes()) {
                                eprintln!("Failed to send command: {}", e);
                            }
                            // Check if the command is the same as the last command in the history
                            if self.command_history.is_empty()
                                || *self.command_history.last().unwrap() != self.command
                            {
                                // Add the command to the history only if it's different from the last command
                                self.command_history.push(self.command.clone());
                            }
                            self.current_history_index = self.command_history.len(); // Reset the index to the end of the history
                            self.command.clear();
                        } else {
                            self.command.push(' ');
                            println!("Command is empty"); // Debug print
                        }
                        // Request focus for the TextEdit
                        response.request_focus();
                    }
                });
            });
        });

        let mut open = *self.show_connection_prompt.borrow();
        let mut close_window = false;

        if open {
            egui::Window::new("Connect to Telnet Server")
                .open(&mut self.show_connection_prompt.borrow_mut())
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
                // Request a repaint for the next frame
                // println!("Received data: {}", data.text());
            }
            ctx.request_repaint();
        }
        self.window_resize_test.show(ctx);
        self.telnet_client.show(ctx);
    }
}
