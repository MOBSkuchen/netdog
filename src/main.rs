mod errors;
mod threading;
mod request;
mod response;
mod system;
mod logger;

use std::{fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}};
use crate::errors::DogError;
use crate::logger::Logger;
use crate::system::System;
use crate::request::{HttpRequest};
use crate::threading::ThreadPool;

const VERSION: &str = env!("CARGO_PKG_VERSION");


pub fn make_link(prefix: &str, addr: &str) -> String {
    format!("{}://\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\", prefix, addr, addr)
}

struct NetDog {
    system: System,
    listener: TcpListener,
    pool: ThreadPool,
}

impl NetDog {
    pub fn new(cfg_file_path: String) -> Self {
        let system_r = System::from_file(cfg_file_path);
        if system_r.is_err() { DogError::__terminate(); }
        let system = system_r.unwrap();

        let addr = format!("{}:{:?}", system.ip, system.port);
        println!("Running on {}", make_link("http", addr.as_str()));

        let listener = TcpListener::bind(addr).unwrap();
        let pool = ThreadPool::new(&system.logger, system.max_cons as usize);

        Self { system, listener, pool}
    }

    fn start(&self) {
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            let sys = self.system.clone();
            let log = self.system.logger.clone();
            self.pool.execute(|| {
                NetDog::handle_connection(stream, sys, log);
            });
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
            system.route_error(request_r.unwrap_err()).send(logger, &stream);
        } else {
            system.route(request_r.unwrap()).send(logger, &stream);
        }
    }
}

fn main() {
    println!("Netdog v{VERSION} - by MOBSkuchen");
    let args = std::env::args().collect::<Vec<String>>();
    let mut config_path = "config.toml".to_string();
    // I know that this is suboptimal, but I hope that LLVM will optimize it for me
    if args.len() > 1 && fs::exists(args[1].clone()).unwrap() {
        config_path = args[1].clone();
    } else if args.len() > 1 && args[1].clone().to_lowercase() == "help" {
        println!("Not enough help? More at {}", make_link("https", "github.com/MOBSkuchen/netdog"));
        println!("Usage | ´netdog´ or ´netdog <my-config.toml>´");
        println!("  OR  | ´netdog help´ or ´netdog version´");
        println!("If you don't specify your config file path, it will default to 'config.toml'");
        return;
    } else if args.len() > 1 && args[1].clone().to_lowercase() == "version" {
        return;
    } else {
        println!("Tip: The default config file is config.toml\nUse ´netdog <my-config.toml>´ to specify your own");
    }
    let netdog = NetDog::new(config_path);
    netdog.start();
}
