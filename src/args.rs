// Parse command line arguments
// Supports individual short switches like gentup -o -b -f
// Supports clustered shorts like gentup -obf
// Supports long switches like gentup --version
// Supports mixed shorts and longs, like gentup --optional -f -ob

use crate::VERSION;
use std::env::{self, Args};

// Define a single command line argument
#[derive(Debug)]
pub struct ArgumentStruct {
    short: String, // Short command line switches like -o
    long: String,  // Long command line switches like --optional
    desc: String,  // A description so we can generate the -help output
    switch: bool,  // Store the on/off state of the command line switch
}

// Define a vector of command line arguments
pub type ArgV = Vec<ArgumentStruct>;

// Define methods of searching the argument vector
pub trait Search {
    fn contains(&self, supplied: &str) -> bool;
    fn setflag(&mut self, flag: &char);
    fn setflag_from_long(&mut self, flag: String);
    fn getflag(&self, flag: &str) -> bool;
    fn help(&self) -> String;
    fn usage(&self) -> String;
    fn version() -> String;
    fn parse(args: Args) -> Result<Self, String>
    where
        Self: Sized;
}

// Define the functions for the methods for a command line argument
impl ArgumentStruct {
    // Construct an ArgumentStruct from separate input variables
    fn from(short: &str, long: &str, desc: &str) -> Self {
        ArgumentStruct {
            short: short.to_string(),
            long: long.to_string(),
            desc: desc.to_string(),
            switch: false,
        }
    }

    // We only have to add new command line switch options in this part of the code. No other
    // modifications to the rest of the arg.rs file should be required
    // Build a Vector of ArgumentStructs that this project accepts as valid command line syntax
    fn build() -> ArgV {
        let mut valid_args = vec![ArgumentStruct::from(
            "b",
            "background",
            "Perform source fetching in the background during update",
        )];
        valid_args.push(ArgumentStruct::from(
            "c",
            "cleanup",
            "Perform cleanup tasks after a successful upgrade",
        ));
        valid_args.push(ArgumentStruct::from(
            "f",
            "force",
            "Force eix-sync, bypassing the timestamp check",
        ));
        valid_args.push(ArgumentStruct::from(
            "h",
            "help",
            "Display this help text, then exit",
        ));
        valid_args.push(ArgumentStruct::from(
            "o",
            "optional",
            "Install optional packages from /etc/default/gentup",
        ));
        valid_args.push(ArgumentStruct::from(
            "t",
            "trim",
            "Perform an fstrim after the upgrade",
        ));
        valid_args.push(ArgumentStruct::from(
            "u",
            "unattended",
            "Unattended upgrade - only partially implemented",
        ));
        valid_args.push(ArgumentStruct::from(
            "V",
            "version",
            "Display the program version",
        ));
        valid_args
    }
}

impl Search for ArgV {
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
    fn getflag(&self, flag: &str) -> bool {
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

    // parse() is a private function dealing with private variables that are not exposed to the
    // calling code. The only hooks exposed to calling code is the ArgV::parse() method. The only
    // thing the calling code has to worry about is if the returning Result enum is Ok or Err.
    // Ok means the user-supplied command line arguments made sense.
    // Err means the user-supplied command line arguments were syntactically incorrect
    //
    // If the returning Result is Ok, the calling code can then call methods on the Vector like
    // .getflag("--force") which will return true if the flag was set by the user. Neat
    //
    // Adding new command line flags as the project is modified now barely requires any logic
    // changes to the argument parsing code.
    fn parse(args: Args) -> Result<Self, String> {
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
        let mut valid_args: ArgV = ArgumentStruct::build();
        let mut first = true;
        for arg in args {
            if first {
                first = false;
                continue;
            }
            match &arg[..] {
                "-h" | "--help" => {
                    return Err(Self::help(&valid_args));
                }
                "-V" | "--version" => {
                    return Err(Self::version());
                }
                supplied => {
                    if supplied.contains("--") {
                        if valid_args.contains(supplied) {
                            valid_args.setflag_from_long(supplied.to_string());
                        } else {
                            return Err(Self::usage(&valid_args));
                        }
                    } else {
                        for individual in supplied.chars() {
                            if individual.eq(&'-') {
                                continue;
                            }
                            if valid_args.contains(&(individual.to_string())) {
                                valid_args.setflag(&individual);
                            } else {
                                return Err(Self::usage(&valid_args));
                            }
                        }
                    }
                }
            }
        }
        Ok(valid_args)
    }
}
