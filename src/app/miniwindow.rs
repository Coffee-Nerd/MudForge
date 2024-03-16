use egui::{TextEdit, Window};

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct WindowResizeTest {
    auto_sized_open: bool,
    resizable_scroll_open: bool,
    resizable_embedded_scroll_open: bool,
    resizable_without_scroll_open: bool,
    resizable_with_text_edit_open: bool,
    freely_resized_open: bool,
    text: String,
}

impl WindowResizeTest {
    pub fn new() -> Self {
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

    pub fn show(&mut self, ctx: &egui::Context) {
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
