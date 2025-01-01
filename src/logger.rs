use std::fs::{File, OpenOptions};
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

#[derive(Debug, Clone)]
pub struct Logger {
    write_file: Option<String>,
    do_print: bool,
    deactivated: bool,
}

impl Logger {
    pub fn new(do_print: bool, out_file: Option<String>) -> DogResult<Self> {
        let file = if out_file.is_some() {
            let x = File::open(out_file.clone().unwrap());
            if x.is_ok() {Some(out_file.unwrap())}
            else {return Err(DogError::new("usr-fileopen-log".into(), "Could not open log file".to_string()))}
        } else { None };
        Ok(Self { write_file: file, do_print, deactivated: false})
    }

    fn __write_out(&mut self, s: &str) {
        if self.write_file.clone().is_some() {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(self.write_file.clone().unwrap())
                .unwrap();

            let res = writeln!(file, "{}", s);
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