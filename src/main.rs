mod errors;
mod logger;
mod request;
mod response;
mod script;
mod system;
mod threading;
mod clparser;

use crate::errors::DogError;
use crate::logger::Logger;
use crate::request::HttpRequest;
use crate::system::System;
use crate::threading::ThreadPool;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use colorize_rs::AnsiColor;
use crate::clparser::{fetch_args_clean, Argument, ArgumentParser, Flag};

fn set_thread_panic_hook() {
    use std::panic::{set_hook, take_hook};
    let orig_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        orig_hook(panic_info);
        DogError::new(
            &Logger::default(),
            "netpup-panic".to_string(),
            panic_info.to_string(),
        )
        .print();
    }));
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn make_link(prefix: &str, addr: &str) -> String {
    format!(
        "{}://\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\",
        prefix, addr, addr
    )
}

struct NetDog {
    system: System,
    listener: TcpListener,
    pool: ThreadPool,
}

impl NetDog {
    pub fn new(cfg_file_path: String) -> Self {
        let system_r = System::from_file(cfg_file_path);
        if system_r.is_err() {
            DogError::__terminate();
        }
        let system = system_r.unwrap();

        let addr = format!("{}:{:?}", system.ip, system.port);
        println!("Running on {}", make_link("http", addr.as_str()));

        let listener = TcpListener::bind(addr).unwrap();
        let pool = ThreadPool::new(&system.logger, system.max_cons as usize);

        Self {
            system,
            listener,
            pool,
        }
    }

    fn start(&mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let sys = self.system.clone();
                    let log = self.system.logger.clone();
                    self.pool.execute(|| {
                        NetDog::handle_connection(stream, sys, log);
                    });
                }
                Err(_e) => {
                    self.system.logger.info("Connection failed");
                }
            }
        }
    }

    fn handle_connection(stream: TcpStream, mut system: System, logger: Logger) {
        let buf_reader = BufReader::new(&stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let request_r = HttpRequest::from_raw(http_request);
        if request_r.is_err() {
            system
                .route_error(request_r.unwrap_err())
                .send(&logger, &stream);
        } else {
            system.route(request_r.unwrap()).send(&logger, &stream);
        }
    }
}

fn _netpup_start(_: &ArgumentParser, args: &Vec<String>) -> bool {
    let config_path = args[0].clone();
    println!("{} ({}) >> {}", "netpup".bold().underlined().b_magenta(),
             ("v".to_string() + VERSION).underlined().faint(),
             "starting...".bold().b_yellow());
    
    let mut netpup = NetDog::new(config_path);
    netpup.start();
    
    true
}

fn main() {
    set_thread_panic_hook();
    
    let mut argument_parser = ArgumentParser::new();
    argument_parser.add_help();
    argument_parser.add_version();
    argument_parser.add_no_color();
    
    argument_parser.add_argument(Argument::new("start".to_string(), vec![], mk_clfn!(_netpup_start), 
                                       "Starts Netpup".to_string(),
                                       false));

    argument_parser.add_flag(Flag::new("--config-path".to_string(), "-c".to_string(),
                                       true, empty!(),
                                       "Set config file path, defaults to 'config.toml'".to_string()));
    
    let result = argument_parser.parse(fetch_args_clean(), true);
    if result.is_err() { argument_parser.handle_errors(result.unwrap_err()); return; }
    let (pending_calls, flag_map) = result.unwrap();
    
    for pending_call in pending_calls {
        if pending_call.has_name("start".to_string()) {
            pending_call.call(&argument_parser, Some(&vec![(&flag_map).get("--config-path").unwrap().clone().or(Some("config.toml".to_string())).unwrap()]));
            break
        }
        
        if pending_call.call(&argument_parser, None) {break}
    }
}
