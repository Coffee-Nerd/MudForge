use std::cell::RefCell;
pub mod ansi_color;
pub mod functions;
mod lua_execution;
mod miniwindow;
mod settings_window;
use settings_window::SettingsWindow;
mod styles;
pub mod telnet;
use crate::app::lua_execution::LuaExecutor;
use egui::{Color32, Layout, TextStyle};
use miniwindow::WindowResizeTest;
use mlua::Lua;
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
    show_connection_prompt: RefCell<bool>,
    show_settings: RefCell<bool>,
    #[serde(skip)]
    settings_window: SettingsWindow,
    ip_address: String,
    port: String,
    command: String,
    command_history: Vec<String>,
    current_history_index: usize,
    fps: f64,
    #[serde(skip)]
    last_frame_time: Option<Instant>,
    #[serde(skip)]
    frame_durations: VecDeque<f64>,
    show_lua_execution_window: RefCell<bool>,
    #[serde(skip)]
    lua: Option<Lua>,
    #[serde(skip)]
    lua_executor: LuaExecutor,
    lua_code: String,
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set custom style and fonts
        let style = styles::default_style();
        cc.egui_ctx.set_style(style);
        let font = styles::custom_font();
        cc.egui_ctx.set_fonts(font);
        let lua_executor = LuaExecutor::new().expect("Failed to initialize Lua executor");

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
            window_resize_test: WindowResizeTest::new(),
            telnet_client: telnet::TelnetClient::new(),
            show_connection_prompt: RefCell::new(false),
            show_settings: RefCell::new(false),
            settings_window: SettingsWindow::default(),
            ip_address: "127.0.0.1".to_owned(),
            port: 23.to_string(),
            command: String::new(),
            command_history: Vec::new(),
            current_history_index: 0,
            fps: 0.0,
            last_frame_time: None,
            frame_durations: VecDeque::with_capacity(10),
            show_lua_execution_window: RefCell::new(false),
            lua: None,
            lua_executor,
            lua_code: String::new(),
        }
    }
}

impl eframe::App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_menu(ctx);
        self.update_ui(ctx);
        self.handle_telnet_input();
        self.update_fps();
        ctx.request_repaint();
    }
}

impl TemplateApp {
    fn update_menu(&mut self, ctx: &egui::Context) {
        let menus: &[(&str, Vec<(&str, Box<dyn Fn(&mut Self, &egui::Context)>)>)] = &[
            (
                "File",
                vec![
                    (
                        "Settings",
                        Box::new(|s, _| {
                            *s.show_settings.borrow_mut() = true;
                            s.settings_window.open = true;
                        }),
                    ),
                    (
                        "Quit",
                        Box::new(|_, ctx| ctx.send_viewport_cmd(egui::ViewportCommand::Close)),
                    ),
                ],
            ),
            (
                "Connection",
                vec![(
                    "New",
                    Box::new(|s, _| {
                        s.show_connection_prompt.replace(true);
                    }),
                )],
            ),
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
    }

    fn update_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    egui::warn_if_debug_build(ui);
                    ui.label(format!("FPS: {:.0}", self.fps));
                });
                ui.set_max_width(ui.available_size().x);
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    let input_box_width = ui.available_size().x - 100.0;

                    let response = ui.add_sized(
                        [input_box_width, ui.text_style_height(&TextStyle::Body)],
                        |ui: &mut egui::Ui| ui.text_edit_singleline(&mut self.command),
                    );

                    self.handle_lua_execution_window(ctx, ui);

                    self.handle_command_input(ui, response);
                });
            });
        });

        self.handle_connection_prompt(ctx);
        self.settings_window.show(ctx);
        self.window_resize_test.show(ctx);
        self.telnet_client.show(ctx);
    }

    fn handle_telnet_input(&mut self) {
        if self.telnet_client.is_connected() {
            if let Some(_data) = self.telnet_client.read_nonblocking() {}
        }
    }

    fn update_fps(&mut self) {
        let now = Instant::now();
        if let Some(last_frame_time) = self.last_frame_time {
            let elapsed = now.duration_since(last_frame_time).as_secs_f64();
            self.frame_durations.push_front(elapsed);
            if self.frame_durations.len() > 10 {
                self.frame_durations.pop_back();
            }
        }
        self.last_frame_time = Some(now);

        if let Some(last_update_time) = self.last_frame_time {
            if now.duration_since(last_update_time).as_secs() >= 1 {
                let total_duration: f64 = self.frame_durations.iter().sum();
                self.fps = 10.0 / total_duration;
                self.last_frame_time = Some(now);
            }
        } else {
            self.last_frame_time = Some(now);
        }
    }

    fn handle_lua_execution_window(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        if ctx.input(|i| i.key_down(egui::Key::Escape) && i.key_pressed(egui::Key::I)) {
            let mut open = self.show_lua_execution_window.borrow_mut();
            *open = !*open;
        }

        let open = *self.show_lua_execution_window.borrow();
        if open {
            let app = self;
            egui::Window::new("Lua Execution")
                .open(&mut app.show_lua_execution_window.borrow_mut())
                .show(ctx, |ui| {
                    ui.label("Enter Lua code to execute:");
                    ui.add_space(8.0);
                    ui.code_editor(&mut app.lua_code);
                    ui.add_space(8.0);
                    if ui.button("Execute").clicked() {
                        if let Err(err) = app.lua_executor.execute(&app.lua_code) {
                            let error_message = format!("Error executing Lua code: {}\n", err);
                            app.telnet_client.append_text(&error_message, Color32::RED);
                        } else {
                            let output = app.lua_executor.take_output();
                            app.telnet_client.append_text(&output, Color32::KHAKI);
                        }
                    }
                });
        }
    }

    fn handle_command_input(&mut self, ui: &mut egui::Ui, response: egui::Response) {
        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            if self.current_history_index > 0 {
                self.current_history_index -= 1;
                self.command = self.command_history[self.current_history_index]
                    .trim()
                    .to_string();
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

        if ui.button("Send").clicked()
            || response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
        {
            if !self.command.is_empty() {
                //        println!("Sending command: {}", self.command); // Add debug log here
                self.command.push('\n');
                if let Err(e) = self.telnet_client.write(self.command.as_bytes()) {
                    eprintln!("Failed to send command: {}", e);
                }
                if self.command_history.is_empty()
                    || *self.command_history.last().unwrap() != self.command
                {
                    self.command_history.push(self.command.clone());
                }
                self.current_history_index = self.command_history.len();
                self.command.clear();
            } else {
                self.command.push(' ');
                println!("Command is empty");
            }
            response.request_focus();
        }
    }

    fn handle_connection_prompt(&mut self, ctx: &egui::Context) {
        let open = *self.show_connection_prompt.borrow();
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
                        ui.text_edit_singleline(&mut self.port);
                    });
                    if ui.button("Connect").clicked() {
                        if let Err(e) = self.telnet_client.connect(&self.ip_address, &self.port) {
                            eprintln!("Connection error: {}", e);
                        }
                        close_window = true;
                    }
                });

            if close_window {
                *self.show_connection_prompt.borrow_mut() = false;
            }
        }
    }
}
