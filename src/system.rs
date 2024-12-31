use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use regex::Regex;
use serde::Deserialize;
use toml::{Table, Value};
use crate::errors::{DogError, DogResult, NetError, NetResult};
use crate::errors::HttpCode::{INTERNAL_ERROR, NOT_FOUND};
use crate::request::{Headers, Methods};
use crate::response::{ContentType, HttpResponse};

fn unwrap_or_error<T>(results: Vec<Option<T>>) -> Option<Vec<T>> {
    let mut unwrapped = Vec::new();

    for result in results {
        match result {
            Some(value) => unwrapped.push(value),
            None => return None, // Return the first error encountered
        }
    }

    Some(unwrapped)
}

fn map_url(route: &Route, url: &str) -> NetResult<Route> {
    let regex = Regex::new(&route.url).map_err(|_| NetError::new(NOT_FOUND, Some("Resource not found".to_string())))?;
    if regex.is_match(url) {
        Ok(route.clone())
    } else {
        Err(NetError::new(NOT_FOUND, Some("Resource not found".to_string())))
    }
}

fn map_url_multiple(routes: &[Route], url: &str) -> NetResult<Route> {
    for route in routes {
        let x = map_url(route, url);
        if x.is_ok() {return Ok(x.unwrap());}
    }
    Err(NetError::new(NOT_FOUND, Some("No matching route found".to_string())))
}

#[derive(Deserialize)]
struct Config_toml {
    pub ip: String,
    pub port: Option<u16>,
    pub max_cons: Option<u32>,
    pub routes: Table,
    pub errors: Table,
}

#[derive(Clone, Debug)]
pub struct Route {
    name: String,
    path: String,
    url: String,
    methods: Vec<Methods>
}

impl Route {
    pub fn new(name: String, t: Table) -> DogResult<Self> {
        if !t.contains_key("path") {return Err(DogError::new("usr-cfgensure-cfgld".to_string(), "Missing key 'path'".to_string()))}
        if !t.contains_key("url") {return Err(DogError::new("usr-cfgensure-cfgld".to_string(), "Missing key 'url'".to_string()))}
        let methods = 
            if t.contains_key("method") { 
            Methods::from_str(t.get("method").unwrap().as_str().unwrap())
                .map(|t1| {vec![t1]}).
                or_else(|e| {
                    return Err(DogError::new("usr-cfgensure-cfgld".to_string(), "Ill formatted key 'method'".to_string()))
                }) }
            else if t.contains_key("methods") {
                let v = unwrap_or_error(
                    t.get("methods").unwrap().as_array().unwrap().iter().map(|t1| {t1.as_str()}).collect::<Vec<Option<&str>>>());
                if v.is_none() {
                    return Err(DogError::new("usr-cfgensure-cfgld".to_string(), "Ill formatted key 'methods'".to_string()))
                }
                Methods::from_str_mult(v.unwrap()).or_else(|e1|
                    Err(DogError::new("usr-cfgensure-cfgld".to_string(), "Ill formatted key 'methods'".to_string()))
                )
            } else { return Err(DogError::new("usr-cfgensure-cfgld".to_string(), "Missing key 'method' or 'methods'".to_string())) };
        
        Ok(Self {name, path: t.get("path").unwrap().to_string(), url: t.get("url").unwrap().to_string(), methods: methods?})
    }
    
    pub fn tbljob(t: Table) -> DogResult<HashMap<String, Route>> {
        let mut hm = HashMap::new();
        for x in t.keys() {
            hm.insert(x.to_string(), Route::new(x.to_string(), t.get(x).unwrap().as_table().unwrap().clone())?);
        }
        Ok(hm)
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct System {
    pub ip: String,
    pub port: u16,
    pub max_cons: u32,
    pub routes: HashMap<String, Route>,
    pub errors: HashMap<u16, ErrorRoute>,
}

impl System {
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
        if config_r.is_err() {return Err::<System, DogError>(DogError::new("usr-toml-cfgld".to_string(), format!("Config file at '{}' could not be parsed", path)))}
        
        let config = config_r.unwrap();
        
        Ok(System::new(config)?)
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
    
    pub fn route_error(&self, error: NetError) -> HttpResponse {
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