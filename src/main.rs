mod errors;
mod logger;
mod request;
mod response;
mod script;
mod system;
mod threading;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

use crate::errors::DogError;
use crate::logger::Logger;
use crate::request::HttpRequest;
use crate::system::System;
use crate::threading::ThreadPool;
use std::{env, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}};
use std::process::{exit, Command};
use clap::{Arg, ColorChoice};

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

fn _netpup_start(config_path: String) {
    println!("netpup (v{}) >> starting...", VERSION);
    
    let mut netpup = NetDog::new(config_path);
    netpup.start();
}

fn update_and_restart() {
    println!("Updating {}...", NAME);
    let status = Command::new("cargo")
        .args(["install", NAME, "--force"])
        .status()
        .expect("Failed to update package");

    if status.success() {
        let mut args: Vec<String> = env::args().collect();
        args.pop();
        args.push("--no-update".to_string());
        let exe = env::current_exe().expect("Failed to get current executable");
        Command::new(exe).args(args).spawn().expect("Failed to restart");
        exit(0);
    } else {
        eprintln!("Update failed.");
    }
}

fn get_latest_version() -> String {
    let output = Command::new("cargo")
        .args(["search", NAME])
        .output()
        .expect("Failed to search for package");

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(line) = stdout.lines().find(|l| l.starts_with(NAME)) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        return parts[2].trim_matches('"').to_string();
    }
    String::new()
}

fn main() {
    set_thread_panic_hook();

    let matches = clap::Command::new(NAME)
        .about(DESCRIPTION)
        .version(VERSION)
        .color(ColorChoice::Never)
        .arg(Arg::new("start")
            .long("start")
            .short('s')
            .help("Runs netpup")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-update")
            .long("nu")
            .long("no-update")
            .help("Prevents automatic updates using cargo")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("config-path")
            .long("config-path")
            .short('c')
            .help("Prevents automatic updates using cargo")
            .value_hint(clap::ValueHint::FilePath)
            .action(clap::ArgAction::Set))
        .arg(Arg::new("version")
            .short('v')
            .long("version")
            .help("Displays the version")
            .action(clap::ArgAction::Version))
        .get_matches();

    if !matches.get_flag("no-update") && VERSION != get_latest_version() {
        update_and_restart()
    }

    let config_path =
        if let Some(config_path) =
        matches.get_one::<String>("config-path") { config_path }
        else { "config.toml" };

    if matches.get_flag("start") {
        _netpup_start(config_path.to_string())
    }
}
