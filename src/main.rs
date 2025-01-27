mod errors;
mod logger;
mod request;
mod response;
mod script;
mod system;
mod threading;

use crate::errors::DogError;
use crate::logger::Logger;
use crate::request::HttpRequest;
use crate::system::System;
use crate::threading::ThreadPool;
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

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

fn main() {
    set_thread_panic_hook();
    println!("Netdog v{VERSION} - by MOBSkuchen");
    let args = std::env::args().collect::<Vec<String>>();
    let mut config_path = "config.toml";
    // I know that this is suboptimal, but I hope that LLVM will optimize it for me
    if args.len() > 1 && fs::exists(&args[1]).unwrap() {
        config_path = &args[1];
    } else if args.len() > 1 && args[1].to_lowercase() == "help" {
        println!(
            "Not enough help? More at {}",
            make_link("https", "github.com/MOBSkuchen/netdog")
        );
        println!("Usage | ´netdog´ or ´netdog <my-config.toml>´");
        println!("  OR  | ´netdog help´ or ´netdog version´");
        println!("If you don't specify your config file path, it will default to 'config.toml'");
        return;
    } else if args.len() > 1 && args[1].to_lowercase() == "version" {
        return;
    } else if args.len() > 1 && !fs::exists(&args[1]).unwrap() {
        println!("Tip: The default config file is config.toml\nUse ´netdog <my-config.toml>´ to specify your own");
    }
    let mut netdog = NetDog::new(config_path.to_string());
    netdog.start();
}
