use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use mlua;
use mlua::{Function, Lua, ObjectLike, StdLib, Table, UserData};
use mlua::prelude::LuaError;
use crate::errors::{DogError, DogResult, HttpCode};
use crate::logger::Logger;
use crate::request::HttpRequest;
use crate::response::{ContentType, HttpResponse};

impl UserData for HttpRequest {}
impl UserData for HttpResponse {}

#[derive(Clone, Debug)]
pub struct Script {
    path: String,
    function: Function,
}

impl Script {
    pub fn new(lua: &Lua, path: String) -> Result<Self, ()> {
        let src = fs::read(&path);
        if src.is_err() {return Err(())}
        let function = lua.load(src.unwrap()).set_name(&path).into_function().unwrap();
        Ok(Self {path, function})
    }
    
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self.function.call::<()>(())?)
    }
}

#[derive(Clone, Debug)]
pub struct ScriptLoader {
    lua: Arc<Mutex<Lua>>,
    scripts: HashMap<String, Script>,
    logger: Logger
}

unsafe impl Send for ScriptLoader {}

fn every<T, I>(v: I) -> bool
where
    I: IntoIterator<Item = T>,
    T: std::ops::Not<Output = bool>,
{
    v.into_iter().all(|x| !!x)
}

fn _lua_read(lua: &Lua, path: String) -> Result<mlua::String, LuaError> {
    fs::read_to_string(path).map(|t| {lua.convert(t).unwrap()}).or(Err(LuaError::runtime("Unable to read file")))
}

fn _lua_write(lua: &Lua, (path, content): (String, String)) -> Result<(), LuaError> {
    fs::write(path, content).or(Err(LuaError::runtime("Unable to read file")))
}

fn _mk_logger(lua: &Lua) -> Result<Logger, LuaError> {
    let globals = lua.globals();
    let logger_file: Result<Option<String>, ()> = globals.get("__logger_file").and_then(|t| {Ok(Some(t))}).or_else(|e| {Ok(None)});
    let logger = Logger::new(globals.get("__logger_print")?, logger_file.unwrap()).unwrap();
    Ok(logger)
}

fn _lua_log_info(lua: &Lua, msg: String) -> Result<(), LuaError> {
    let mut logger = _mk_logger(lua)?;
    logger.info(msg.as_str());
    Ok(())
}

fn _lua_log_error(lua: &Lua, msg: String) -> Result<(), LuaError> {
    let mut logger = _mk_logger(lua)?;
    logger.info(msg.as_str());
    Ok(())
}

fn _lua_log_fatal(lua: &Lua, msg: String) -> Result<(), LuaError> {
    let mut logger = _mk_logger(lua)?;
    logger.info(msg.as_str());
    Ok(())
}

impl ScriptLoader {
    pub fn new(logger: Logger, script_locs: HashMap<String, String>) -> DogResult<Self> {
        let lua = Lua::new();
        lua.sandbox(true).unwrap();
        let globals = lua.globals();
        if (&logger).write_file.is_some() {
            globals.set("__logger_file", logger.write_file.clone().unwrap().to_owned()).expect("Panic on Lua globals init");
        }
        globals.set("__logger_print", logger.do_print).expect("Panic on Lua globals init");
        globals.set("read".to_string(), lua.create_function(_lua_read).unwrap()).expect("Panic on Lua globals init");
        globals.set("write".to_string(), lua.create_function(_lua_write).unwrap()).expect("Panic on Lua globals init");
        globals.set("log_info".to_string(), lua.create_function(_lua_log_info).unwrap()).expect("Panic on Lua globals init");
        globals.set("log_error".to_string(), lua.create_function(_lua_log_error).unwrap()).expect("Panic on Lua globals init");
        globals.set("log_fatal".to_string(), lua.create_function(_lua_log_fatal).unwrap()).expect("Panic on Lua globals init");
        let mut scripts = HashMap::new();
        
        for script_loc in script_locs {
            let script = Script::new(&lua, script_loc.1);
            if script.is_err() {
                return Err(DogError::new(logger.clone(), "usr-scripts-ensloc".to_string(), "Could not ensure that all scripts exist".to_string()));
            }
            scripts.insert(script_loc.0, script.unwrap());
        }
        
        Ok(Self {lua: Arc::new(Mutex::new(lua)), logger, scripts})
    }
    
    pub fn table_to_response(&self, table: Table) -> DogResult<HttpResponse> {
        if !table.contains_key("code").unwrap() {
            return Err(DogError::new(self.logger.clone(), "usr-scripts-evres".to_string(), "Missing 'code' in response table".to_string()))
        }
        if !table.contains_key("resp").unwrap() {
            return Err(DogError::new(self.logger.clone(), "usr-scripts-evres".to_string(), "Missing 'resp' in response table".to_string()))
        }
        if !table.contains_key("headers").unwrap() {
            return Err(DogError::new(self.logger.clone(), "usr-scripts-evres".to_string(), "Missing 'headers' in response table".to_string()))
        }
        if !table.contains_key("content").unwrap() {
            return Err(DogError::new(self.logger.clone(), "usr-scripts-evres".to_string(), "Missing 'content' in response table".to_string()))
        }
        
        let code = HttpCode::from_num(table.get("code").unwrap());
        if code.is_none() { return Err(DogError::new(self.logger.clone(), "usr-scripts-evres".to_string(), "Malformed entry 'code' in response table".to_string())) }
        let content: String = table.get("content").unwrap();
        let ct: String = table.get("type").unwrap();
        
        Ok(HttpResponse::new((code.unwrap(), table.get("resp").unwrap()),
                             table.get("headers").unwrap(), 
                             (content.into_bytes(), ContentType::from_ext(ct.as_str()))))
    }
    
    pub fn run_script(&self, script: &str, request: HttpRequest) -> DogResult<HttpResponse> {
        let result = self.scripts.get(script).unwrap().function.call::<Table>(request);
        if result.is_err() {
            return Err(DogError::new(self.logger.clone(), "usr-script-run".to_string(), format!("Running script failed => {}", result.unwrap_err())))
        }
        Ok(self.table_to_response(result.unwrap())?)
    }
}