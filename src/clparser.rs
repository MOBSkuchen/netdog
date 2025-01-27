use std::collections::HashMap;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

use colorize_rs::{color_enabled, AnsiColor};
use crate::clparser::ClParserError::{FlagMissingValue, TooFewArguments, TooManyArguments};

#[derive(Debug)]
pub enum ClParserError {
    TooFewArguments(usize),
    TooManyArguments(usize),
    FlagMissingValue(String)
}

pub type ClParserResult<T> = Result<T, ClParserError>;
pub type ClParserResultCallQueue = ClParserResult<Vec<PendingCall>>;
pub type CallFunction = Box<dyn Fn(&ArgumentParser, &Vec<String>) -> bool>;

#[macro_export] 
macro_rules! mk_clfn {
    ($function: ident) => {
        Box::new(|apr, args| {$function(apr, args)})
    };
}

#[macro_export]
macro_rules! mk_clfn_ng {
    ($function: ident) => {
        Box::new(|apr, _| {$function(apr)})
    };
}

#[macro_export]
macro_rules! mk_clfn_static {
    ($function: ident) => {
        Box::new(|_, _| {$function()})
    };
}

#[macro_export]
macro_rules! empty {
    () => {
        Box::new(|_, _| {false})
    };
}

pub fn fetch_args_clean() -> Vec<String> {
    let args: Vec<String> = std::env::args().collect();
    args[1..].to_vec()
}

fn _print_help(argument_parser: &ArgumentParser) -> bool {
    let mut shorts: Vec<String> = argument_parser.positionals.iter().map(|x| {x.short()}).collect();
    shorts.append(&mut argument_parser.arguments.iter().map(|x| {x.short()}).collect());
    shorts.append(&mut argument_parser.flags.iter().map(|x| {x.short()}).collect());
    println!("{} ({}) usage => {}", argument_parser.prog.clone().bold().underlined().b_magenta(), ("v".to_string() + &*argument_parser.version.clone()).underlined().faint(), shorts.join(" "));
    for argument in &argument_parser.arguments {
        println!("-> {}", argument.get_description())
    }

    for pos in &argument_parser.positionals {
        println!("-> {}", pos.get_description())
    }

    for flag in &argument_parser.flags {
        println!("-> {}", flag.get_description())
    }
    
    true
}

fn _print_version(argument_parser: &ArgumentParser) -> bool {
    println!("{} ({}) : {}", argument_parser.prog.clone().bold().underlined().b_magenta(), 
             ("v".to_string() + &*argument_parser.version.clone()).underlined().faint(), 
             argument_parser.description.clone().bold().underlined().b_yellow());
    true
}

fn _disable_color(_: &ArgumentParser) -> bool {
    color_enabled(false);
    false
}

pub struct Argument {
    name: String,
    nargs: usize,
    args: Vec<String>,
    triggers: CallFunction,
    positional: bool,
    description: String,
}

impl Argument {
    pub fn new(name: String, args: Vec<String>, triggers: CallFunction, description: String, positional: bool) -> Self {
        let nargs = args.len();
        Self {
            name,
            nargs,
            args,
            triggers,
            positional,
            description,
        }
    }

    fn get_description(&self) -> String {
        if self.nargs < 2 {
            format!("{} | {}", self.name.clone().bold().b_green(), self.description.clone().bold())
        } else {
            format!("{} => {} | {}", self.name.clone().bold().b_green(), self.args.join(" ").yellow(), self.description.clone().bold())
        }
    }

    fn short(&self) -> String {
        if self.nargs < 2 {
            format!("[{}]", self.name.clone().bold().b_green())
        } else {
            format!("[{} {}]", self.name.clone().bold().b_green(), self.args.join(" ").yellow())
        }
    }

    fn call(&self, argument_parser: &ArgumentParser, args: &Vec<String>) -> bool {
        (self.triggers)(argument_parser, args)
    }
}

pub struct Flag {
    name: String,
    mini: String,
    value: bool,
    triggers: CallFunction,
    description: String,
}

impl Flag {
    pub fn new(name: String, mini: String, value: bool, triggers: CallFunction, description: String) -> Self {
        Self {
            name,
            mini,
            value,
            triggers,
            description
        }
    }

    fn get_description(&self) -> String {
        if self.value {
            format!("{} / {} => {} | {}", self.name.clone().bold().b_blue(), self.mini.clone().bold().b_cyan(),
                    "...".to_string().yellow(), self.description.clone().bold())
        } else {
            format!("{} / {} | {}", self.name.clone().bold().b_blue(), self.mini.clone().bold().b_cyan(), self.description.clone().bold())
        }
    }

    fn short(&self) -> String {
        if self.value {
            format!("[{} / {} {}]", self.name.clone().bold().b_blue(), self.mini.clone().bold().b_cyan(), "<VALUE>".to_string().bold().yellow())
        } else {
            format!("[{} / {}]", self.name.clone().bold().b_blue(), self.mini.clone().bold().b_cyan())
        }
    }

    fn call(&self, argument_parser: &ArgumentParser, value: &Vec<String>) -> bool {
        (self.triggers)(argument_parser, value)
    }
}

#[derive(Debug)]
pub enum CallType {
    ARGUMENT,
    POSITIONAL,
    FLAG
}

#[derive(Debug)]
pub struct PendingCall {
    name: String,
    index: usize,
    args: Vec<String>,
    call_type: CallType
}

impl PendingCall {
    pub fn new(name: String, index: usize, args: Vec<String>, call_type: CallType) -> Self {
        Self { name, index, args, call_type}
    }
    
    pub fn has_name(&self, name: String) -> bool {
        self.name == name
    }
    
    pub fn call(&self, argument_parser: &ArgumentParser, spec_args: Option<&Vec<String>>) -> bool {
        let args = spec_args.or(Some(&self.args));
        match self.call_type {
            CallType::ARGUMENT => {
                argument_parser.arguments[self.index].call(argument_parser, args.unwrap())
            }
            CallType::POSITIONAL => {
                argument_parser.positionals[self.index].call(argument_parser, args.unwrap())
            }
            CallType::FLAG => {
                argument_parser.flags[self.index].call(argument_parser, args.unwrap())
            }
        }
    }
}

pub struct ArgumentParser {
    prog: String,
    version: String,
    description: String,
    arguments: Vec<Argument>,
    positionals: Vec<Argument>,
    flags: Vec<Flag>,
}

impl ArgumentParser {
    pub fn new() -> Self {
        Self {
            prog: NAME.to_string(),
            version: VERSION.to_string(),
            description: DESCRIPTION.to_string(),
            arguments: Vec::new(),
            positionals: Vec::new(),
            flags: Vec::new(),
        }
    }

    pub fn add_argument(&mut self, arg: Argument) -> &mut Self {
        if arg.positional {
            self.positionals.push(arg);
        } else {
            self.arguments.push(arg);
        }
        self
    }

    pub fn add_flag(&mut self, flag: Flag) -> &mut Self {
        self.flags.push(flag);
        self
    }

    fn parse_argument(&self, argument: &Argument, args: &Vec<String>) -> ClParserResult<Vec<String>> {
        if args.len() < (argument.nargs+1) {
            Err(TooFewArguments(argument.nargs))
        } else {
            Ok(args[1..argument.nargs+1].to_vec())
        }
    }

    fn parse_positionals(&self, args: &mut Vec<String>) -> ClParserResultCallQueue {
        let mut remove_list = vec![];
        let mut pending_calls = vec![];
        for (i, pos) in self.positionals.iter().enumerate() {
            pending_calls.push(PendingCall::new(pos.name.clone(), i, self.parse_argument(pos, args)?, CallType::POSITIONAL));
            remove_list.append(&mut (i..pos.args.len()+1).collect());
        }

        for (i, remove) in remove_list.iter().enumerate() {
            args.remove(remove - i);
        }
        
        Ok(pending_calls)
    }

    fn parse_arguments(&self, args: &mut Vec<String>) -> ClParserResultCallQueue {
        let mut remove_list = vec![];
        let mut pending_calls = vec![];
        for (i, arg) in args.iter().enumerate() {
            for (n, darg) in (&self.arguments).iter().enumerate() {
                if darg.name == *arg {
                    pending_calls.push(PendingCall::new(darg.name.clone(), n, self.parse_argument(darg, args)?, CallType::ARGUMENT));
                    remove_list.append(&mut (i..darg.args.len()+1).collect());
                }
            }
        }

        for (i, remove) in remove_list.iter().enumerate() {
            args.remove(remove - i);
        }
        
        Ok(pending_calls)
    }

    fn parse_flag(&self, flag: &Flag, item: &str, args: &Vec<String>, i: usize) -> ClParserResult<Vec<String>> {
        let val = if flag.value {
            let value;
            if item.contains("=") {
                value = item.split_once("=").expect("This should not fail").1;
            } else if (args.len() as i32 - 1) > (i as i32) {
                value = &args[i + 1];
            } else {
                return Err(FlagMissingValue(item.into()))
            }
            vec![value.to_string()]
        } else {
            vec![]
        };

        Ok(val)
    }

    fn parse_flags(&self, flag_map: &mut HashMap<String, Option<String>>, args: &mut Vec<String>) -> ClParserResultCallQueue {
        let mut remove_list = vec![];
        let mut pending_calls = vec![];
        
        for i in 0..args.len() {
            let item = &args[i];
            for (n, flag) in (&self.flags).iter().enumerate() {
                if item.starts_with(&flag.name) && (!flag.value || (&flag.name == item)) || 
                    item.starts_with(&flag.mini) && (!flag.value || (&flag.mini == item)) {
                    let rgs = self.parse_flag(flag, item, args, i)?;
                    flag_map.insert(flag.name.clone(), if flag.value {Some((&rgs[0]).clone())} else {None});
                    pending_calls.push(PendingCall::new(flag.name.clone(), n, rgs, CallType::FLAG));
                    remove_list.push(i);
                    if flag.value { remove_list.push(i + 1); }
                }
            }
        }

        for (i, remove) in remove_list.iter().enumerate() {
            args.remove(remove - i);
        }

        Ok(pending_calls)
    }

    pub fn parse(&self, mut args: Vec<String>, auto_help_nargs: bool) -> ClParserResult<(Vec<PendingCall>, HashMap<String, Option<String>>)> {
        let mut pending_calls = vec![];
        let mut flag_map = HashMap::new();

        for flag in &self.flags {
            flag_map.insert(flag.name.clone(), None);
        }
        
        if auto_help_nargs && args.len() == 0 {
            _print_help(&self);
            return Ok((pending_calls, flag_map));
        }
        
        pending_calls.append(&mut self.parse_flags(&mut flag_map, &mut args)?);
        pending_calls.append(&mut self.parse_arguments(&mut args)?);
        pending_calls.append(&mut self.parse_positionals(&mut args)?);
        
        if args.len() > 0 {
            Err(TooManyArguments(args.len()))
        } else {
            Ok((pending_calls, flag_map))
        }
    }
    
    pub fn handle_errors(&self, error: ClParserError) {
        match error {
            TooFewArguments(expected) => {
                eprintln!("{}\n", format!("Invalid usage! Got too few arguments. Expected {}", expected).b_red().bold());
                _print_help(self);
            }
            TooManyArguments(got) => {
                eprintln!("{}\n", format!("Invalid usage! Got too many arguments. Got {} too many", got).b_red().bold());
                _print_help(self);
            }
            FlagMissingValue(flag) => {
                eprintln!("{}\n", format!("The flag '{}' is missing its value! Look at its description below:", flag).b_red().bold());
                _print_help(self);
            }
        }
    }

    pub fn get_auto_help(&self) -> Flag {
        Flag::new(
            "--help".to_string(),
            "-h".to_string(),
            false,
            mk_clfn_ng!(_print_help),
            "Get help".to_string())
    }

    pub fn get_auto_version(&self) -> Flag {
        Flag::new(
            "--version".to_string(),
            "-v".to_string(),
            false,
            mk_clfn_ng!(_print_version),
            "Get version".to_string())
    }

    pub fn get_auto_no_color(&self) -> Flag {
        Flag::new(
            "--no-color".to_string(),
            "-nc".to_string(),
            false,
            mk_clfn_ng!(_disable_color),
            "Disable colors (please use as first argument / flag)".to_string())
    }

    pub fn add_help(&mut self) -> &mut Self {
        self.flags.push(self.get_auto_help());
        self
    }

    pub fn add_version(&mut self) -> &mut Self {
        self.flags.push(self.get_auto_version());
        self
    }

    pub fn add_no_color(&mut self) -> &mut Self {
        self.flags.push(self.get_auto_no_color());
        self
    }
}