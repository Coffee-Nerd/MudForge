use egui::{Color32, Ui, Visuals, Window};

#[derive(PartialEq)]
pub enum SettingsCategory {
    Style,
    Appearance,
}

impl Default for SettingsCategory {
    fn default() -> Self {
        SettingsCategory::Style
    }
}

pub struct SettingsWindow {
    pub selected_category: SettingsCategory,
    pub style_settings: StyleSettings,
    pub appearance_settings: AppearanceSettings,
    pub open: bool,
}

impl Default for SettingsWindow {
    fn default() -> Self {
        Self {
            selected_category: SettingsCategory::default(),
            style_settings: StyleSettings::default(),
            appearance_settings: AppearanceSettings::default(),
            open: false,
        }
    }
}

impl SettingsWindow {
    pub fn show(&mut self, ctx: &egui::Context) {
        Window::new("Settings")
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Side panel equivalent for selecting categories
                    ui.vertical(|ui| {
                        ui.selectable_value(
                            &mut self.selected_category,
                            SettingsCategory::Style,
                            "Style",
                        );
                        ui.selectable_value(
                            &mut self.selected_category,
                            SettingsCategory::Appearance,
                            "Appearance",
                        );
                    });

                    ui.separator();

                    // Display settings for the selected category
                    match self.selected_category {
                        SettingsCategory::Style => self.style_settings.ui(ui),
                        SettingsCategory::Appearance => self.appearance_settings.ui(ui),
                        // Handle UI for other categories...
                    }
                });
            });
    }
}

#[derive(Default)]
pub struct StyleSettings {
    pub visuals: Visuals, // We are directly using egui's Visuals struct here
}

impl StyleSettings {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Visuals");
            ui.add_space(10.0);
            let visuals = &mut self.visuals;
            ui.horizontal(|ui| {
                ui.color_edit_button_srgba(&mut visuals.window_fill);
                ui.label("Window background");
            });
            // Add other visuals settings here...
        });
    }
}

#[derive(Default)]
pub struct AppearanceSettings {
    pub primary_color: Color32,
    // Add other appearance-related settings fields here...
}

impl AppearanceSettings {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Appearance");
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.color_edit_button_srgba(&mut self.primary_color);
                ui.label("Primary color");
            });
            // Add other appearance settings here...
        });
    }
}

// Add other settings structs and their impls here...
