use crate::app::functions::init_lua;
use crate::app::telnet::TelnetClient;
use mlua::{Lua, Result};
use std::env;
use std::fs;
use std::sync::{Arc, Mutex};
pub struct LuaExecutor {
    lua: Lua,
    output_buffer: Arc<Mutex<String>>,
}

impl Default for LuaExecutor {
    fn default() -> Self {
        Self::new().expect("Failed to initialize Lua executor")
    }
}

impl LuaExecutor {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        let output_buffer = Arc::new(Mutex::new(String::new()));
        let telnet_client = Arc::new(Mutex::new(TelnetClient::new())); // Create a new TelnetClient instance
        init_lua(&lua, telnet_client)?; // Call init_lua to expose custom functions

        // Get the current working directory
        let current_dir = env::current_dir().unwrap();
        let lua_dir = current_dir.join("lua");

        // Set LUA_PATH to include the lua directory
        let lua_path = format!("{}/?.lua", lua_dir.to_str().unwrap());

        // Set the LUA_PATH in the Lua state
        lua.load(&format!(r#"package.path = "{}""#, lua_path))
            .exec()?;

        // Load Lua scripts from the "lua" folder
        load_lua_scripts(&lua, "lua")?;

        Ok(Self { lua, output_buffer })
    }

    pub fn execute(&self, code: &str) -> Result<()> {
        let modified_lua_code = format!(
            "local old_print = print; \
             local old_color_print = color_print; \
             local old_note = Note; \
             local old_tell = Tell; \
             local old_colour_note = colour_note; \
             local old_colour_tell = ColourTell; \
             local old_ansi_note = AnsiNote; \
             local output = ''; \
             print = function(...) old_print(...); output = output .. table.concat({{...}}, ' ') .. '\\n'; end; \
             color_print = function(...) old_color_print(...); output = output .. table.concat({{...}}, ' ') .. '\\n'; end; \
             Note = function(...) old_note(...); output = output .. table.concat({{...}}, ' ') .. '\\n'; end; \
             Tell = function(...) old_tell(...); output = output .. table.concat({{...}}, ' '); end; \
             colour_note = function(...) old_colour_note(...); output = output .. table.concat({{...}}, ' ') .. '\\n'; end; \
             ColourTell = function(...) old_colour_tell(...); output = output .. table.concat({{...}}, ' '); end; \
             AnsiNote = function(...) old_ansi_note(...); output = output .. table.concat({{...}}, ' ') .. '\\n'; end; \
             {} \
             print = old_print; \
             color_print = old_color_print; \
             Note = old_note; \
             Tell = old_tell; \
             colour_note = old_colour_note; \
             ColourTell = old_colour_tell; \
             AnsiNote = old_ansi_note; \
             return output",
            code
        );

        let output = self.lua.load(&modified_lua_code).eval::<String>()?;
        *self.output_buffer.lock().unwrap() = output;
        Ok(())
    }
    pub fn take_output(&self) -> String {
        self.output_buffer.lock().unwrap().clone()
    }
}

fn load_lua_scripts(lua: &Lua, lua_folder: &str) -> mlua::Result<()> {
    let paths = match fs::read_dir(lua_folder) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Failed to read Lua directory: {}", e);
            return Ok(());
        }
    };

    for path in paths {
        let path = match path {
            Ok(path) => path.path(),
            Err(e) => {
                eprintln!("Failed to read path in Lua directory: {}", e);
                continue;
            }
        };

        if path.extension().and_then(|ext| ext.to_str()) == Some("lua") {
            let script = match fs::read_to_string(&path) {
                Ok(script) => script,
                Err(e) => {
                    eprintln!("Failed to read Lua file {:?}: {}", path, e);
                    continue;
                }
            };

            if let Err(e) = lua.load(&script).exec() {
                eprintln!("Error executing Lua script {:?}: {}", path, e);
            }
        }
    }

    Ok(())
}
