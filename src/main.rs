#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;

use app::functions::init_lua;
use app::telnet::TelnetClient;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct CustomError(String);

impl From<LuaError> for CustomError {
    fn from(error: LuaError) -> Self {
        CustomError(error.to_string())
    }
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CustomError {}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let lua = Lua::new();
    let telnet_client = Arc::new(Mutex::new(TelnetClient::new()));
    init_lua(&lua, telnet_client).map_err(CustomError::from)?;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .unwrap(),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "MudForge",
        native_options,
        Box::new(|cc| Box::new(app::TemplateApp::new(cc))),
    )
    .map_err(|e| Box::new(CustomError(e.to_string())) as Box<dyn std::error::Error>)
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(app::TemplateApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
