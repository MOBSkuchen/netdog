use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::str::FromStr;
use serde::Deserialize;
use toml::{Table};
use crate::errors::{DogError, DogResult, HttpCode, NetError, NetResult};
use crate::errors::HttpCode::{INTERNAL_ERROR, NOT_FOUND};
use crate::logger::Logger;
use crate::request::{Headers, HttpRequest, Methods};
use crate::response::{ContentType, HttpResponse};
use crate::response::ContentType::HTML;
use crate::script::{Script, ScriptLoader};

fn unwrap_or_error<T>(results: Vec<Option<T>>) -> Option<Vec<T>> {
    let mut unwrapped = Vec::new();

    for result in results {
        match result {
            Some(value) => unwrapped.push(value),
            None => return None,
        }
    }

    Some(unwrapped)
}

fn url_resolve(route: &Route, url: &str, method: &Methods) -> Result<Route, ()> { 
    let resolved_path = if route.url.contains("*") {
        let parts: Vec<&str> = route.url.split('*').collect();
        if parts.len() > 2 { return Err(()); }
        if !url.starts_with(parts[0]) || !url.ends_with(parts.get(1).unwrap_or(&"")) { return Err(()); }
        let dynamic_part = &url[parts[0].len()..url.len() - parts.get(1).unwrap_or(&"").len()];
        if !route.methods.contains(method) { return Err(()); }
        route.path.replace('*', dynamic_part)
    } else {
        if route.url != url {
            return Err(())
        }
        route.path.clone()
    };
    Ok(Route {
        path: resolved_path,
        url: route.url.clone(),
        methods: route.methods.clone(),
        name: (&route).name.clone().into(),
        path_is_script: route.path_is_script
    })
}

fn url_resolve_mult(routes: &[Route], url: &str, method: Methods) -> NetResult<Route> {
    for route in routes {
        let x = url_resolve(route, url, &method);
        if x.is_ok() {return Ok(x.unwrap());}
    }
    Err(NetError::new(NOT_FOUND, Some("No matching route found".to_string())))
}

#[derive(Deserialize)]
struct Config_toml {
    pub ip: String,
    pub port: Option<u16>,
    pub max_cons: Option<u32>,
    pub logger: Option<LoggerCfg>,
    pub routes: Table,
    pub errors: Option<Table>,
    pub scripts: Option<Table>
}

#[derive(Deserialize)]
struct LoggerCfg {
    print: Option<bool>,
    log_file: Option<String>
}

impl LoggerCfg {
    pub fn default() -> Self {
        Self {print: Some(true), log_file: None}
    }
}

#[derive(Clone, Debug)]
pub struct Route {
    name: String,
    path: String,
    url: String,
    methods: Vec<Methods>,
    path_is_script: bool
}

impl Route {
    pub fn new(logger: Logger, name: String, t: Table) -> DogResult<Self> {
        let (path, path_is_script) = if t.contains_key("path") {
            (t.get("path").unwrap().as_str().unwrap().to_string(), false)
        } else if t.contains_key("script") {
            (t.get("script").unwrap().as_str().unwrap().to_string(), true)
        } else {
            return Err(DogError::new(logger, "usr-cfgensure-cfgld".to_string(), "Missing key 'path' or 'script'".to_string()))
        };
        if !t.contains_key("url") {return Err(DogError::new(logger, "usr-cfgensure-cfgld".to_string(), "Missing key 'url'".to_string()))}
        let methods =
            if t.contains_key("method") {
            Methods::from_str(t.get("method").unwrap().as_str().unwrap())
                .map(|t1| {vec![t1]}).
                or_else(|_e| {
                    return Err(DogError::new(logger, "usr-cfgensure-cfgld".to_string(), "Ill formatted key 'method'".to_string()))
                }) }
            else if t.contains_key("methods") {
                let v = unwrap_or_error(
                    t.get("methods").unwrap().as_array().unwrap().iter().map(|t1| {t1.as_str()}).collect::<Vec<Option<&str>>>());
                if v.is_none() {
                    return Err(DogError::new(logger, "usr-cfgensure-cfgld".to_string(), "Ill formatted key 'methods'".to_string()))
                }
                Methods::from_str_mult(v.unwrap()).or_else(|_e1|
                    Err(DogError::new(logger, "usr-cfgensure-cfgld".to_string(), "Ill formatted key 'methods'".to_string()))
                )
            } else { return Err(DogError::new(logger, "usr-cfgensure-cfgld".to_string(), "Missing key 'method' or 'methods'".to_string())) };

        Ok(Self {name, path, url: t.get("url").unwrap().as_str().unwrap().to_string(), methods: methods?, path_is_script})
    }
    
    pub fn tbljob(logger: Logger, t: Table) -> DogResult<HashMap<String, Route>> {
        let mut hm = HashMap::new();
        for x in t.keys() {
            hm.insert(x.to_string(), Route::new(logger.clone(), x.to_string(), t.get(x).unwrap().as_table().unwrap().clone())?);
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
    pub fn new(logger: Logger, erc: u16, t: Table) -> DogResult<Self> {
        if !t.contains_key("path") {return Err(DogError::new(logger, "usr-cfgensure-cfgld".to_string(), "Missing key 'path'".to_string()))}

        Ok(Self {erc, path: t.get("path").unwrap().to_string()})
    }

    pub fn tbljob(logger: Logger, t: Table) -> DogResult<HashMap<u16, ErrorRoute>> {
        let mut hm = HashMap::new();
        for x in t.keys() {
            let erc = u16::from_str(x).unwrap();
            hm.insert(erc, ErrorRoute::new(logger.clone(), erc, t.get(x).unwrap().as_table().unwrap().clone())?);
        }
        Ok(hm)
    }
}

#[derive(Debug, Clone)]
pub struct System {
    pub ip: String,
    pub port: u16,
    pub max_cons: u32,
    pub routes: HashMap<String, Route>,
    pub errors: HashMap<u16, ErrorRoute>,
    pub logger: Logger,
    pub script_loader: ScriptLoader
}

impl System {
    pub fn new(cfg_t: Config_toml) -> DogResult<Self> {
        let logger_cfg = cfg_t.logger.or_else(|| {Some(LoggerCfg::default())}).unwrap();
        let logger = Logger::new(logger_cfg.print.is_some_and(|t| {t}), logger_cfg.log_file.or_else(|| {None}))?;
        let errors = if cfg_t.errors.is_some() {
            ErrorRoute::tbljob(logger.clone(), cfg_t.errors.unwrap())?
        } else {
            HashMap::new()
        };
        let mut scripts = HashMap::new();
        if cfg_t.scripts.is_some() {
            for script in cfg_t.scripts.unwrap() {
                let sr = script.1.as_table();
                if sr.is_none() {
                    return Err(DogError::new(logger, "usr-cfgensure-cfgld".to_string(), "Could not parse scripts".to_string()))
                }
                let s = sr.unwrap();
                if !s.contains_key("path") {
                    return Err(DogError::new(logger, "usr-cfgensure-cfgld".to_string(), "Missing key 'path'".to_string()))
                }
                let l = s.get("path").unwrap().as_str();
                if l.is_none() {
                    return Err(DogError::new(logger, "usr-cfgensure-cfgld".to_string(), "Ill formatted key 'path'".to_string()))
                }
                scripts.insert(script.0, l.unwrap().to_string());
            }
        }
        Ok(Self {
            ip: cfg_t.ip,
            port: cfg_t.port.unwrap_or_else(|| { 8080 }),
            max_cons: cfg_t.max_cons.unwrap_or_else(|| { 100 }),
            routes: Route::tbljob(logger.clone(), cfg_t.routes)?,
            errors,
            logger: logger.clone(),
            script_loader: ScriptLoader::new(logger, scripts)?
        })
    }
    
    pub fn from_file(path: String) -> DogResult<Self> {
        let file_contents_r = fs::read_to_string(&path);
        if file_contents_r.is_err() {return Err(DogError::fatal(Logger::default(), "usr-fileread-cfgld".to_string(), format!("Could not read file '{}'", path)))}
        let config_r = toml::from_str(file_contents_r.unwrap().as_str());
        if config_r.is_err() {return Err::<System, DogError>(DogError::fatal(Logger::default(), "usr-toml-cfgld".to_string(), format!("Config file at '{}' could not be parsed", path)))}
        
        let config = config_r.unwrap();
        
        Ok(System::new(config)?)
    }

    pub fn netdog_error(&mut self, error: DogError) -> HttpResponse {
        self.logger.error(format!("Serving client with NetDog error [{}]", error.__fmtx()).as_str());
        HttpResponse::new((INTERNAL_ERROR, error.name), Headers::new(), (vec![], ContentType::NONE))
    }

    pub fn load_content_path(&self, path: String) -> DogResult<Vec<u8>> {
        let r = fs::read(&path);
        if r.is_err() {
            Err(DogError::new(self.logger.clone(), "usr-fileread-ctserve".to_string(), format!("Could not load user provided resource at {}", path)))
        } else {
            Ok(r.unwrap())
        }
    }

    pub fn route_error(&mut self, error: NetError) -> HttpResponse {
        let erc = &(error.erc.clone() as u16);
        if (&self.errors).contains_key(&erc) {
            let r_fn = self.errors.get(erc).unwrap().path.clone();
            let content = self.load_content_path(r_fn.clone().into());
            if content.is_err() {return self.netdog_error(content.unwrap_err())}
            HttpResponse::new((error.erc, error.details), Headers::new(), (content.unwrap(), ContentType::from_file_name(&*r_fn)))
        } else {
            HttpResponse::new((error.erc, error.details), Headers::new(), (format!("Error {}", erc).into_bytes(), HTML))
        }
    }

    pub fn route_to_response(&mut self, route: Route) -> HttpResponse {
        let content = self.load_content_path(route.path.clone().into());
        if content.is_err() {return self.netdog_error(content.unwrap_err())}
        HttpResponse::new((HttpCode::OK, "OK".to_string()), Headers::new(), (content.unwrap(), ContentType::from_file_name(&*route.path)))
    }

    pub fn route(&mut self, req: HttpRequest) -> HttpResponse {
        let response = url_resolve_mult(&self.routes.values().map(|a| {a.to_owned()}).collect::<Vec<Route>>(), &*req.path, req.method.clone());
        if response.is_err() {
            self.logger.info(format!("No route available for < {} > -> responding with error", req.format()).as_str());
            self.route_error(response.unwrap_err())
        }
        else {
            let route = response.unwrap();
            if route.path_is_script {
                let ret = self.script_loader.run_script(&route.name, req);
                return if ret.is_err() {
                    self.logger.error(format!("Got an error from script {}", route.name).as_str());
                    self.netdog_error(ret.unwrap_err())
                } else {
                    ret.unwrap()
                }
            }
            self.logger.info(format!("Routing < {} > -> {}", req.format(), route.path).as_str());
            self.route_to_response(route)
        }
    }
}