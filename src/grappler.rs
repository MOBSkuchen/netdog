use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use serde::Deserialize;
use toml::Table;
use crate::errors::{DogError, DogResult, NetError};
use crate::errors::HttpCode::INTERNAL_ERROR;
use crate::request::Headers;
use crate::response::{ContentType, HttpResponse};

#[derive(Deserialize)]
struct Config_toml {
    pub ip: String,
    pub port: Option<u16>,
    pub max_cons: Option<u32>,
    pub routes: Table,
    pub errors: Table,
}

#[derive(Clone)]
pub struct Route {
    name: String,
    path: String,
    url: String
}

impl Route {
    pub fn new(name: String, t: Table) -> DogResult<Self> {
        if !t.contains_key("path") {return Err(DogError::new("usr-cfgensure-cfgld".to_string(), "Missing key 'path'".to_string()))}
        if !t.contains_key("url") {return Err(DogError::new("usr-cfgensure-cfgld".to_string(), "Missing key 'url'".to_string()))}
        
        Ok(Self {name, path: t.get("path").unwrap().to_string(), url: t.get("url").unwrap().to_string()})
    }
    
    pub fn tbljob(t: Table) -> DogResult<HashMap<String, Route>> {
        let mut hm = HashMap::new();
        for x in t.keys() {
            hm.insert(x.to_string(), Route::new(x.to_string(), t.get(x).unwrap().as_table().unwrap().clone())?);
        }
        Ok(hm)
    }
}

#[derive(Clone)]
pub struct ErrorRoute {
    erc: u16,
    path: String
}


impl ErrorRoute {
    pub fn new(erc: u16, t: Table) -> DogResult<Self> {
        if !t.contains_key("path") {return Err(DogError::new("usr-cfgensure-cfgld".to_string(), "Missing key 'path'".to_string()))}

        Ok(Self {erc, path: t.get("path").unwrap().to_string()})
    }

    pub fn tbljob(t: Table) -> DogResult<HashMap<u16, ErrorRoute>> {
        let mut hm = HashMap::new();
        for x in t.keys() {
            let erc = u16::from_str(x).unwrap();
            hm.insert(erc, ErrorRoute::new(erc, t.get(x).unwrap().as_table().unwrap().clone())?);
        }
        Ok(hm)
    }
}

#[derive(Clone)]
pub struct Config {
    pub ip: String,
    pub port: u16,
    pub max_cons: u32,
    pub routes: HashMap<String, Route>,
    pub errors: HashMap<u16, ErrorRoute>,
}

impl Config {
    pub fn new(cfg_t: Config_toml) -> DogResult<Self> {
        Ok(Self {
            ip: cfg_t.ip,
            port: cfg_t.port.unwrap_or_else(|| { 8080 }),
            max_cons: cfg_t.max_cons.unwrap_or_else(|| { 100 }),
            routes: Route::tbljob(cfg_t.routes)?,
            errors: ErrorRoute::tbljob(cfg_t.errors)?
        })
    }
    
    pub fn from_file(path: String) -> DogResult<Self> {
        let file_contents_r = fs::read_to_string(&path);
        if file_contents_r.is_err() {return Err(DogError::new("usr-fileread-cfgld".to_string(), format!("Could not read file '{}'", path)))}
        let config_r = toml::from_str(file_contents_r.unwrap().as_str());
        if config_r.is_err() {return Err::<Config, DogError>(DogError::new("usr-toml-cfgld".to_string(), format!("Config file at '{}' could not be parsed", path)))}
        
        let config = config_r.unwrap();
        
        Ok(Config::new(config)?)
    }
    
    pub fn netdog_error(error: DogError) -> HttpResponse {
        HttpResponse::new((INTERNAL_ERROR, error.name), Headers::new(), (vec![], ContentType::NONE))
    }
    
    pub fn load_content_path(path: String) -> DogResult<Vec<u8>> {
        let r = fs::read(&path);
        if r.is_err() {
            Err(DogError::new("usr-fileread-ctserve".to_string(), format!("Could not load user provided resource at {}", path)))
        } else {
            Ok(r.unwrap())
        }
    }
    
    pub fn try_rout_error(&self, error: NetError) -> HttpResponse {
        let erc = &(error.erc.clone() as u16);
        if (&self.errors).contains_key(&erc) {
            let content = Self::load_content_path(self.errors.get(erc).unwrap().path.clone());
            if content.is_err() {return Self::netdog_error(content.unwrap_err())}
            HttpResponse::new((error.erc, error.details), Headers::new(), (content.unwrap(), ContentType::HTML))
        } else {
            HttpResponse::new((error.erc, error.details), Headers::new(), (format!("Error {}", erc).into_bytes(), ContentType::HTML))
        }
    }
}