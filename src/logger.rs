use std::fs::File;
use std::io::Write;
use crate::errors::{DogError, DogResult};
extern crate chrono;
use chrono::Local;

const DATE_FORMAT_STR: &'static str = "%Y-%m-%d][%H:%M:%S";

#[derive(Debug)]
pub enum LogLevel {
    INFO,
    ERROR,
    FATAL
}

pub struct Logger {
    write_handle: Option<File>,
    do_print: bool,
    deactivated: bool,
}

impl Logger {
    pub fn new(mode: String, out_file: Option<String>) -> DogResult<Self> {
        let handle = if out_file.is_some() {
            let x = File::open(out_file.unwrap());
            if x.is_ok() {Some(x.unwrap())}
            else {return Err(DogError::new("usr-fileopen-log".into(), "Could not open log file".to_string()))}
        } else { None };
        Ok(Self {write_handle: handle, do_print: mode.to_lowercase() == "dev", deactivated: false})
    }

    fn __write_out(&mut self, s: &str) {
        let hnd = &(self.write_handle);
        if hnd.as_ref().is_some() {
            let res = hnd.as_ref().unwrap().write(s.as_bytes());
            if res.is_err() {
                self.deactivated = true;
                DogError::fatal("fsw-writelg-log1".to_string(), "Can not write log".to_string());
            }
        }
    }

    fn __print_out(&self, s: &str) {
        if self.do_print {
            println!("{}", s)
        }
    }

    pub fn log(&mut self, level: LogLevel, message: &str) {
        if self.deactivated {return}
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
}