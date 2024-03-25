use egui::Window;
use std::cell::RefCell; 
mod telnet;
mod miniwindow;
mod ansi_color;
use miniwindow::WindowResizeTest;
use egui::FontFamily;


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
        // Set the custom style
        let mut style: egui::Style = (*cc.egui_ctx.style()).clone();
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(45, 51, 59); // text box background
        style.visuals.faint_bg_color = egui::Color32::from_rgb(45, 51, 59);
        style.visuals.code_bg_color = egui::Color32::from_rgb(45, 51, 59);
        style.visuals.hyperlink_color = egui::Color32::from_rgb(255, 0, 0);
        style.visuals.window_fill = egui::Color32::from_rgb(0, 0, 0); // menu bg, widget bg
        style.visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10); // entire window bg
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(173, 186, 199));
        //style.visuals.window_corner_radius = 10.0;
        style.visuals.button_frame = true;
        style.visuals.collapsing_header_frame = true;
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(35, 39, 46);
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(0., egui::Color32::from_rgb(173, 186, 199));
        style.visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 51, 59);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(45, 51, 59);
        style.visuals.widgets.open.bg_fill = egui::Color32::from_rgb(45, 51, 59);
        cc.egui_ctx.set_style(style);
    
// Custom font
let font_ReFixedysMono = include_bytes!("data/refixedsys-mono.otf").to_vec();
let mut font = egui::FontDefinitions::default();
font.font_data.insert(
    "ReFixedys Mono".to_string(),
    egui::FontData::from_owned(font_ReFixedysMono),
);
font.families.get_mut(&FontFamily::Monospace).unwrap().insert(0, "ReFixedys Mono".to_string());
font.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "ReFixedys Mono".to_string());
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
        
                // Make the CentralPanel fill the entire width of the window
                ui.set_max_width(ui.available_size().x);
        
                // Add a text input field for the command
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    // Calculate the width of the input box to fill the available width
                    let input_box_width = ui.available_size().x - 100.0; // Adjust the subtraction as needed for your layout
        
                    // Use add_sized to set the size of the TextEdit
                    let response = ui.add_sized([input_box_width, ui.text_style_height(&egui::TextStyle::Body)], |ui: &mut egui::Ui| {
                        ui.text_edit_singleline(&mut self.command)
                    });
        
                    // Add the "Send" button
                    if ui.button("Send").clicked() || response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.command.is_empty() {
                            // Append a newline character if required by the Telnet server
                            self.command.push('\n');
                            if let Err(e) = self.telnet_client.write(self.command.as_bytes()) {
                                eprintln!("Failed to send command: {}", e);
                            }
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
                    // Request a repaint for the next frame
            //    println!("Received data: {}", data.text());
            }
            ctx.request_repaint();
        }
        self.window_resize_test.show(ctx);
        self.telnet_client.show(ctx);
    }
}