use egui::Window;

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct TemplateApp {
    label: String,
    value: f32,
    window_resize_test: WindowResizeTest,
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
            ("Window", vec![
                ("Auto-sized Window", Box::new(|s, ctx| s.window_resize_test.auto_sized_open = true)),
                ("Resizable + Scroll Window", Box::new(|s, ctx| s.window_resize_test.resizable_scroll_open = true)),
                ("Resizable + Embedded Scroll Window", Box::new(|s, ctx| s.window_resize_test.resizable_embedded_scroll_open = true)),
                ("Resizable without Scroll Window", Box::new(|s, ctx| s.window_resize_test.resizable_without_scroll_open = true)),
                ("Resizable with TextEdit Window", Box::new(|s, ctx| s.window_resize_test.resizable_with_text_edit_open = true)),
                ("Freely Resized Window", Box::new(|s, ctx| s.window_resize_test.freely_resized_open = true)),
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
            ui.heading("eframe template");
            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });
            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }
            ui.separator();
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        self.window_resize_test.show(ctx);
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

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct WindowResizeTest {
    auto_sized_open: bool,
    resizable_scroll_open: bool,
    resizable_embedded_scroll_open: bool,
    resizable_without_scroll_open: bool,
    resizable_with_text_edit_open: bool,
    freely_resized_open: bool,
    text: String,
}

impl WindowResizeTest {
    fn new() -> Self {
        Self {
            auto_sized_open: false,
            resizable_scroll_open: false,
            resizable_embedded_scroll_open: false,
            resizable_without_scroll_open: false,
            resizable_with_text_edit_open: false,
            freely_resized_open: false,
            text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_owned(),
        }
    }

    fn show(&mut self, ctx: &egui::Context) {
        use egui::*;

        Window::new("↔ auto-sized")
            .open(&mut self.auto_sized_open)
            .auto_sized()
            .show(ctx, |ui| {
                ui.label("This window will auto-size based on its contents.");
                ui.heading("Resize this area:");
                Resize::default().show(ui, |ui| {
                    ui.label(&self.text);
                });
                ui.heading("Resize the above area!");
            });

        Window::new("↔ resizable + scroll")
            .open(&mut self.resizable_scroll_open)
            .vscroll(true)
            .resizable(true)
            .default_height(300.0)
            .show(ctx, |ui| {
                ui.label(
                    "This window is resizable and has a scroll area. You can shrink it to any size.",
                );
                ui.separator();
                ui.label(&self.text);
            });

        Window::new("↔ resizable + embedded scroll")
            .open(&mut self.resizable_embedded_scroll_open)
            .vscroll(false)
            .resizable(true)
            .default_height(300.0)
            .show(ctx, |ui| {
                ui.label("This window is resizable but has no built-in scroll area.");
                ui.label("However, we have a sub-region with a scroll bar:");
                ui.separator();
                ScrollArea::vertical().show(ui, |ui| {
                    ui.label(&format!("{}\n\n{}", self.text, self.text));
                });
            });

        Window::new("↔ resizable without scroll")
            .open(&mut self.resizable_without_scroll_open)
            .vscroll(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("This window is resizable but has no scroll area. This means it can only be resized to a size where all the contents is visible.");
                ui.label("egui will not clip the contents of a window, nor add whitespace to it.");
                ui.separator();
                ui.label(&self.text);
            });

        Window::new("↔ resizable with TextEdit")
            .open(&mut self.resizable_with_text_edit_open)
            .vscroll(false)
            .resizable(true)
            .default_height(300.0)
            .show(ctx, |ui| {
                ui.label("Shows how you can fill an area with a widget.");
                ui.add_sized(ui.available_size(), TextEdit::multiline(&mut self.text));
            });

        Window::new("↔ freely resized")
            .open(&mut self.freely_resized_open)
            .vscroll(false)
            .resizable(true)
            .default_size([250.0, 150.0])
            .show(ctx, |ui| {
                ui.label("This window has empty space that fills up the available space, preventing auto-shrink.");
                ui.vertical_centered(|ui| {
                    ui.label("Powered by egui and eframe.");
                });
                ui.allocate_space(ui.available_size());
            });
    }
}
