// Parse command line arguments
// Supports individual short switches like gentup -o -b -f
// Supports clustered shorts like gentup -obf
// Supports long switches like gentup --version
// Supports mixed shorts and longs, like gentup --optional -f -ob

use crate::VERSION;
use std::env::{self, Args};

// Define an API for checking command line arguments
// Currently there are 3 hooks into the API
// 1. ArgCheck::parse() which parses the command line arguments that the user supplied
// 2. A method on ArgCheck called .get() which returns true if a named command line option was set by the user
// 3. ArgumentStruct::from() that can be used to construct a new command line option

// Define a single command line argument
#[derive(Debug)]
pub struct ArgumentStruct {
    short: String, // Short command line switches like -o
    long: String,  // Long command line switches like --optional
    desc: String,  // A description so we can generate the -help output
    switch: bool,  // Store the on/off state of the command line switch
}

// Define a vector of command line arguments
pub type ArgCheck = Vec<ArgumentStruct>;

// Define methods of searching the argument vector
pub trait Search {
    fn contains(&self, supplied: &str) -> bool;
    fn setflag(&mut self, flag: &char);
    fn setflag_from_long(&mut self, flag: String);
    fn get(&self, flag: &str) -> bool;
    fn help(&self) -> String;
    fn usage(&self) -> String;
    fn version() -> String;
    fn parse(self, args: Args) -> Result<Self, String>
    where
        Self: Sized;
}

// Define the functions for the methods for a command line argument
impl ArgumentStruct {
    // Construct an ArgumentStruct from separate input variables
    pub fn from(short: &str, long: &str, desc: &str) -> Self {
        ArgumentStruct {
            short: short.to_string(),
            long: long.to_string(),
            desc: desc.to_string(),
            switch: false,
        }
    }
}

impl Search for ArgCheck {
    // Return true if the user supplied command line switch is found to be correct
    // The search finds matches for both short and long command line switches
    fn contains(&self, supplied: &str) -> bool {
        for argsearch in self {
            let stripped = supplied.replace('-', "");
            if argsearch.short.eq(&stripped) || argsearch.long.eq(&stripped) {
                return true;
            }
        }
        false
    }

    // Set a command line switch for a particular short flag to true
    fn setflag(&mut self, flag: &char) {
        for argsearch in self {
            if argsearch.short.chars().next().unwrap_or(' ').eq(flag) {
                argsearch.switch = true;
            }
        }
    }

    // Set a command line switch for a particular long flag to true
    fn setflag_from_long(&mut self, flag: String) {
        let stripped = flag.replace('-', "");
        for argsearch in self {
            if argsearch.long.eq(&stripped) {
                argsearch.switch = true;
            }
        }
    }

    // Get the command line switch setting for a named long flag
    fn get(&self, flag: &str) -> bool {
        for argsearch in self {
            if argsearch.long.eq(&flag) {
                return argsearch.switch;
            }
        }
        false
    }

    // Display program help - the user asked for help
    fn help(&self) -> String {
        let mut retval = "Usage:\ngentup [options]\n".to_string();
        for eacharg in self {
            let line = format!(
                "-{:1}, --{:15}\t{}\n",
                eacharg.short, eacharg.long, eacharg.desc
            );
            retval = retval + &line;
        }
        retval
    }

    // Display usage. One of the command line arguments was incorrect
    fn usage(&self) -> String {
        let mut retval = "Error: usage - gentup [".to_string();
        for eacharg in self {
            let line = format!("--{}|", eacharg.long);
            retval = retval + &line;
        }
        retval = format!("{}]", retval);
        retval
    }

    // Display the program version.
    fn version() -> String {
        format!("gentup version {}", VERSION)
    }

    // The parse function is exposed as an API to calling code. It takes a Vector of valid syntax
    // and the user supplied command line arguments. When it has parsed the args it returns a
    // Result. Ok means the user-supplied command line arguments made sense.
    // Err means the user-supplied command line arguments were syntactically incorrect
    //
    // If the returning Result is Ok, the calling code can then call methods on the Vector like
    // .get("--force") which will return true if the flag was set by the user. Neat
    //
    // Adding new command line flags as the project is modified/extended will not require any logic
    // changes to the argument parsing code.
    //
    fn parse(mut self, args: Args) -> Result<Self, String> {
        // Check we are root
        match env::var("USER") {
            Ok(val) => {
                if val != "root" {
                    return Err("You need to be root to run this".to_string());
                }
            }
            Err(_) => {
                return Err("You need to be root to run this".to_string());
            }
        }
        let mut first = true;
        for arg in args {
            if first {
                first = false;
                continue;
            }
            match &arg[..] {
                "-h" | "--help" => {
                    return Err(Self::help(&self));
                }
                "-V" | "--version" => {
                    return Err(Self::version());
                }
                supplied => {
                    if supplied.contains("--") {
                        if self.contains(supplied) {
                            self.setflag_from_long(supplied.to_string());
                        } else {
                            return Err(Self::usage(&self));
                        }
                    } else {
                        for individual in supplied.chars() {
                            if individual.eq(&'-') {
                                continue;
                            }
                            if self.contains(&(individual.to_string())) {
                                self.setflag(&individual);
                            } else {
                                return Err(Self::usage(&self));
                            }
                        }
                    }
                }
            }
        }
        Ok(self)
    }
}
