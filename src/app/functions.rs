use crate::app::telnet::TelnetClient;
use egui::Color32;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

pub struct LuaFunctions {
    telnet_client: Arc<Mutex<TelnetClient>>,
}

impl LuaFunctions {
    pub fn print(&self, text: String) -> LuaResult<()> {
        println!("this is being called");
        let mut telnet_client = self.telnet_client.lock().unwrap();
        telnet_client.append_text(&text, Color32::WHITE);
        Ok(())
    }

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
    println!("Initializing Lua environment with custom print functions...");
    println!("Lua instance address in init_lua: {:p}", lua);

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
    println!("Custom color_print function set in Lua environment.");

    println!("Lua environment initialized successfully.");

    Ok(())
}
