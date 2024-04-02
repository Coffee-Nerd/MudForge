use crate::app::ansi_color::COLOR_MAP;
use crate::app::telnet::parse_ansi_codes;
use crate::app::telnet::TelnetClient;
use egui::Color32;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};
#[derive(Clone)]
pub struct LuaFunctions {
    telnet_client: Arc<Mutex<TelnetClient>>,
}

impl LuaFunctions {
    // PRINTING FUNCTIONS ===========================================================================
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

    pub fn note(&self, text: String) -> LuaResult<()> {
        let mut telnet_client = self.telnet_client.lock().unwrap();
        telnet_client.append_text(&format!("{}\n", text), Color32::WHITE);
        Ok(())
    }

    pub fn tell(&self, text: String) -> LuaResult<()> {
        let mut telnet_client = self.telnet_client.lock().unwrap();
        telnet_client.append_text(&text, Color32::WHITE);
        Ok(())
    }

    pub fn colour_note(
        &self,
        (text_colour, back_colour, text): (String, String, String),
    ) -> LuaResult<()> {
        let text_colour = COLOR_MAP
            .get(&*text_colour)
            .unwrap_or(&Color32::WHITE)
            .clone();
        let back_colour = COLOR_MAP
            .get(&*back_colour)
            .unwrap_or(&Color32::BLACK)
            .clone();
        let mut telnet_client = self.telnet_client.lock().unwrap();
        telnet_client.append_text_with_colours(&format!("{}\n", text), text_colour, back_colour);
        Ok(())
    }

    pub fn colour_tell(
        &self,
        (text_colour, back_colour, text): (String, String, String),
    ) -> LuaResult<()> {
        let text_colour = COLOR_MAP
            .get(&*text_colour)
            .unwrap_or(&Color32::WHITE)
            .clone();
        let back_colour = COLOR_MAP
            .get(&*back_colour)
            .unwrap_or(&Color32::BLACK)
            .clone();
        let mut telnet_client = self.telnet_client.lock().unwrap();
        telnet_client.append_text_with_colours(&text, text_colour, back_colour);
        Ok(())
    }

    pub fn ansi_note(&self, text: String) -> LuaResult<()> {
        let mut telnet_client = self.telnet_client.lock().unwrap();
        let parsed_segments = parse_ansi_codes(text.as_bytes().to_vec());
        for segment in parsed_segments {
            for (text, color) in segment {
                telnet_client.append_text(&text, color);
            }
            // Add a new line at the end of each segment if you want to mimic MUSHclient's behavior
            telnet_client.append_text("\n", Color32::WHITE);
        }
        Ok(())
    }
    //================================================================================================
    // COLOUR FUNCTIONS
    pub fn colour_name_to_rgb(&self, name: String) -> LuaResult<i32> {
        let color = COLOR_MAP.get(&*name).unwrap_or(&Color32::WHITE);
        let rgb = (color.r() as i32) << 16 | (color.g() as i32) << 8 | color.b() as i32;
        Ok(rgb)
    }

    pub fn rgb_colour_to_name(&self, colour: i32) -> LuaResult<String> {
        let r = ((colour >> 16) & 0xFF) as u8;
        let g = ((colour >> 8) & 0xFF) as u8;
        let b = (colour & 0xFF) as u8;
        let color = Color32::from_rgb(r, g, b);
        let name = COLOR_MAP
            .iter()
            .find_map(|(key, &val)| {
                if val == color {
                    Some(key.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| format!("rgb({}, {}, {})", r, g, b));
        Ok(name)
    }

    pub fn ansi(&self, code: i16) -> LuaResult<String> {
        let ansi_code = format!("\x1b[{}m", code);
        Ok(ansi_code)
    }
    //================================================================================================
}

pub fn init_lua(lua: &Lua, telnet_client: Arc<Mutex<TelnetClient>>) -> LuaResult<()> {
    println!("Initializing Lua environment with custom functions...");
    println!("Lua instance address in init_lua: {:p}", lua);

    let functions = LuaFunctions {
        telnet_client: telnet_client.clone(),
    };

    let globals = lua.globals();

    // Clone functions for each closure
    let print_functions = functions.clone();
    let color_print_functions = functions.clone();
    let note_functions = functions.clone();
    let tell_functions = functions.clone();
    let colour_note_functions = functions.clone();
    let colour_tell_functions = functions.clone();
    let ansi_note_functions = functions.clone();
    let colour_name_to_rgb_function = functions.clone();
    let rgb_colour_to_name_function = functions.clone();
    let ansi_function = functions.clone();

    // Set print function
    globals.set(
        "print",
        lua.create_function(move |_, text: String| print_functions.print(text))?,
    )?;

    // Set color_print function
    globals.set(
        "color_print",
        lua.create_function(move |_, args: (String, String)| {
            color_print_functions.color_print(args)
        })?,
    )?;

    // Set note function
    globals.set(
        "Note",
        lua.create_function(move |_, text: String| note_functions.note(text))?,
    )?;

    // Set tell function
    globals.set(
        "Tell",
        lua.create_function(move |_, text: String| tell_functions.tell(text))?,
    )?;

    globals.set(
        "ColourNote",
        lua.create_function(move |_, args: (String, String, String)| {
            colour_note_functions.colour_note(args)
        })?,
    )?;

    // Set colour_tell function
    globals.set(
        "ColourTell",
        lua.create_function(move |_, args: (String, String, String)| {
            colour_tell_functions.colour_tell(args)
        })?,
    )?;

    globals.set(
        "AnsiNote",
        lua.create_function(move |_, text: String| ansi_note_functions.ansi_note(text))?,
    )?;

    globals.set(
        "ColourNameToRGB",
        lua.create_function(move |_, name: String| {
            colour_name_to_rgb_function.colour_name_to_rgb(name)
        })?,
    )?;

    globals.set(
        "RGBColourToName",
        lua.create_function(move |_, colour: i32| {
            rgb_colour_to_name_function.rgb_colour_to_name(colour)
        })?,
    )?;

    globals.set(
        "ANSI",
        lua.create_function(move |_, code: i16| ansi_function.ansi(code))?,
    )?;
    println!("Custom functions set in Lua environment.");
    println!("Lua environment initialized successfully.");

    Ok(())
}
