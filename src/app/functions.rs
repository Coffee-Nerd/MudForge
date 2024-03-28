use crate::app::telnet::TelnetClient;
use egui::Color32;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

// Define a struct that holds a reference to TelnetClient
pub struct LuaFunctions {
    telnet_client: Arc<Mutex<TelnetClient>>,
}

impl LuaFunctions {
    // Method for the print function
    pub fn print(&self, text: String) -> LuaResult<()> {
        let mut telnet_client = self.telnet_client.lock().unwrap();
        telnet_client.append_text(&text, Color32::WHITE);
        Ok(())
    }

    // Method for the color_print function
    pub fn color_print(&self, (text, color): (String, String)) -> LuaResult<()> {
        let color = match color.as_str() {
            "red" => Color32::RED,
            "green" => Color32::GREEN,
            "blue" => Color32::BLUE,
            _ => Color32::WHITE,
        };
        let mut telnet_client = self.telnet_client.lock().unwrap();
        telnet_client.append_text(&text, color);
        Ok(())
    }
}

// Function to initialize and expose the functions to Lua
pub fn init_lua(lua: &Lua, telnet_client: Arc<Mutex<TelnetClient>>) -> LuaResult<()> {
    let print_functions = LuaFunctions {
        telnet_client: telnet_client.clone(),
    };
    let color_print_functions = LuaFunctions { telnet_client };

    let globals = lua.globals();

    let print_function = lua.create_function(move |_, text: String| print_functions.print(text))?;
    globals.set("print", print_function)?;

    let color_print_function = lua.create_function(move |_, args: (String, String)| {
        color_print_functions.color_print(args)
    })?;
    globals.set("color_print", color_print_function)?;

    Ok(())
}
