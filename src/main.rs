mod errors;
mod threading;
mod request;
mod response;
mod system;
mod logger;

use std::{fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}};
use crate::system::System;
use crate::request::{HttpRequest};
use crate::threading::ThreadPool;

struct NetDog {
    system: System,
    listener: TcpListener,
    pool: ThreadPool
}

impl NetDog {
    pub fn new(cfg_file_path: String) -> Self {
        let system_r = System::from_file(cfg_file_path);
        if system_r.is_err() {
            (&system_r).to_owned().unwrap_err().__terminate();
        }
        let system = system_r.unwrap();

        let addr = format!("{}:{:?}", system.ip, system.port);
        println!("Running on {}", addr);
        
        let listener = TcpListener::bind(addr).unwrap();
        let pool = ThreadPool::new(system.max_cons as usize);
        
        Self { system, listener, pool}
    }
    
    fn start(&self) {
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            let system = self.system.clone();
            self.pool.execute(|| {
                NetDog::handle_connection(stream, system);
            })
        }
    }
    
    fn handle_connection(stream: TcpStream, system: System) {
        let buf_reader = BufReader::new(&stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let request_r = HttpRequest::from_raw(http_request);
        if request_r.is_err() {
            system.route_error(request_r.unwrap_err()).send(&stream);
        } else {
            system.route(request_r.unwrap()).send(&stream);
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let mut config_path = "config.toml".to_string();
    if args.len() > 1 && fs::exists(args[1].clone()).unwrap() {
        config_path = args[1].clone();
    }
    let netdog = NetDog::new(config_path);
    netdog.start();
}
