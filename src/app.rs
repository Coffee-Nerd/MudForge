use egui::Window;
mod telnet;
mod miniwindow;
use miniwindow::WindowResizeTest;



#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct TemplateApp {
    label: String,
    value: f32,
    window_resize_test: WindowResizeTest,
    #[serde(skip)]
    telnet_client: telnet::TelnetClient,
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
                ("New", Box::new(|s, _| s.telnet_client.connection_open = true)),
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
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.label);
                });
                if ui.button("Increment").clicked() {
                    self.value += 1.0;
                }
            });
        });

        self.window_resize_test.show(ctx);
        self.telnet_client.show(ctx);
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
