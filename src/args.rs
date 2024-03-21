// Parse command line arguments
// Supports individual short switches like -o -b -f
// Supports clustered shorts like -obf
// Supports long switches like --version
// Supports mixed shorts and longs, like --optional -f -ob

use crate::version::VERSION;
use std::env::{self, Args};

// Define a Struct to contain one single command line option definition
//
pub struct ArgumentStruct {
    short: String, // Short command line options like -o
    long: String,  // Long command line options like --optional
    desc: String,  // A description so we can generate the -help output
    switch: bool,  // Store the on/off state of the command line switch
}

// Define a vector of command line options
//
pub type ArgCheck = Vec<ArgumentStruct>;

// Define traits - these are public so that any calling code can use these methods against a
// Vector of valid command line options
//
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

// Define a public constructor so that calling code can easily construct a Vector of valid command
// line options
//
impl ArgumentStruct {
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
    // Return true if the user supplied command line option is found in the command line options
    // Vector. The search finds matches for both short and long command line options
    //
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
    //
    fn setflag(&mut self, flag: &char) {
        for argsearch in self {
            if argsearch.short.chars().next().unwrap_or(' ').eq(flag) {
                argsearch.switch = true;
            }
        }
    }

    // Set a command line switch for a particular long flag to true
    //
    fn setflag_from_long(&mut self, flag: String) {
        let stripped = flag.replace('-', "");
        for argsearch in self {
            if argsearch.long.eq(&stripped) {
                argsearch.switch = true;
            }
        }
    }

    // Get the command line switch setting for a named long flag
    //
    fn get(&self, flag: &str) -> bool {
        for argsearch in self {
            if argsearch.long.eq(&flag) {
                return argsearch.switch;
            }
        }
        false
    }

    // Display program help - the user asked for help
    //
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
    //
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
    //
    fn version() -> String {
        format!("gentup version {}", VERSION)
    }

    // The parse function is public and exposed to the calling code. It takes a Vector of valid
    // command line options and the user supplied command line arguments. When it has parsed the
    // args it returns a Result. Ok means the user-supplied command line arguments made sense.
    // Err means the user-supplied command line arguments were syntactically incorrect
    //
    // If the returning Result is Ok, the calling code can then call methods on the Vector like
    // .get("--force") which will return true if the flag was set by the user.
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
            // The first arg is the name of the binary e.g gentup, so we skip past onto the next argument
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
                    // Handle the long version of the options, which are prefixed with -- e.g
                    // --force
                    if supplied.contains("--") {
                        // The long version of an option has been supplied
                        if self.contains(supplied) {
                            // A valid long option was found
                            // Set the switch for that option to "true"
                            self.setflag_from_long(supplied.to_string());
                        } else {
                            // Syntax error, so return the usage text as part of the error
                            return Err(Self::usage(&self));
                        }
                    } else {
                        // Handle the short version of the options, which are prefixed with one -
                        // character, e.g -f. Also silently ignore the case where the user didn't
                        // bother with the minus sign at all
                        for individual in supplied.chars() {
                            // Iterate through the command line options
                            if individual.eq(&'-') {
                                continue;
                            }
                            if self.contains(&(individual.to_string())) {
                                // A valid command line switch was found. Set the switch for the
                                // option to "true"
                                self.setflag(&individual);
                            } else {
                                // Syntax error, so return the usage text as part of the error
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
