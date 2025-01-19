use crate::errors::{DogError, DogResult};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
extern crate chrono;
use chrono::Local;
use mlua::UserData;
use serde::{Deserialize, Serialize};

const DATE_FORMAT_STR: &'static str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone)]
pub enum LogLevel {
    INFO,
    ERROR,
    FATAL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logger {
    pub write_file: Option<String>,
    pub do_print: bool,
    deactivated: bool,
}

impl UserData for Logger {}

impl Logger {
    pub fn default() -> Self {
        Self {
            write_file: None,
            do_print: true,
            deactivated: false,
        }
    }

    pub fn new(do_print: bool, out_file: Option<String>) -> DogResult<Self> {
        let file = if out_file.is_some() {
            let out_file = out_file.unwrap();
            if fs::exists(&out_file).unwrap() {
                Some(out_file)
            } else {
                let x = fs::write(&out_file, &*vec![]);
                if x.is_ok() {
                    Some(out_file)
                } else {
                    return Err(DogError::fatal(
                        Self::default(),
                        "usr-fileopen-log".into(),
                        "Could not open log file".to_string(),
                    ));
                }
            }
        } else {
            None
        };
        Ok(Self {
            write_file: file,
            do_print,
            deactivated: false,
        })
    }

    fn __write_out(&mut self, s: &str) {
        if self.write_file.is_some() {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(self.write_file.clone().unwrap())
                .unwrap();

            let res = writeln!(file, "{}", s);
            if res.is_err() {
                self.deactivated = true;
                DogError::fatal(
                    self.to_owned(),
                    "fsw-writelg-log1".to_string(),
                    "Can not write log".to_string(),
                );
            }
        }
    }

    fn __print_out(&self, s: &str) {
        if self.do_print {
            println!("{}", s)
        }
    }

    pub fn log(&mut self, level: LogLevel, message: &str) {
        if self.deactivated {
            return;
        }
        let tm = Local::now();
        let fmt_log = format!("{} | {:?} : {}", tm.format(DATE_FORMAT_STR), level, message);
        self.__print_out(&fmt_log);
        self.__write_out(&fmt_log);
    }

    pub fn error(&mut self, message: &str) {
        self.log(LogLevel::ERROR, message);
    }

    pub fn info(&mut self, message: &str) {
        self.log(LogLevel::INFO, message);
    }

    pub fn fatal(&mut self, message: &str) {
        self.log(LogLevel::FATAL, message);
        DogError::__terminate();
    }
}
