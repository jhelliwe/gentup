// Parse command line arguments

use crate::VERSION;
use std::env::{self, Args};

#[derive(Debug)]
pub struct GentupArgs {
    pub cleanup: bool,
    pub force: bool,
    pub separate: bool,
}

pub fn cmdlinargs(args: Args) -> Option<GentupArgs> {
    // Check we are root
    match env::var("USER") {
        Ok(val) => {
            if val != "root" {
                eprintln!("You need to be root to run this");
                return None;
            }
        }
        Err(_) => {
            eprintln!("You need to be root to run this");
            return None;
        }
    }
    // Parse command line arguments
    let mut myargs = GentupArgs {
        cleanup: false,
        force: false,
        separate: false,
    };

    let mut first = true;
    for arg in args {
        if first {
            first = false;
            continue;
        }
        match &arg[..] {
            "-h" | "--help" => {
                println!(
                    "Usage:\n\n \
                    gentup [options]\n \
                    Options:\n\n\
                    -c, --cleanup    Perform cleanup tasks only\n\
                    -f, --force      force eix-sync, bypassing the timestamp check\n\
                    -s  --separate   Perform source fetching separately before update\n\
                    -h, --help       Display this help text, then exit\n\
                    -V, --version    Display the program version\
                "
                );
                return None;
            }
            "-V" | "--version" => {
                println!("gentup version {}", VERSION);
                return None;
            }
            "-f" | "--force" => {
                myargs.force = true;
            }
            "-s" | "--separate" => {
                myargs.separate = true;
            }
            "-c" | "--cleanup" => {
                myargs.cleanup = true;
            }
            _ => {
                eprintln!("Error: usage - gentup [--help|--force|--separate|--cleanup|--version]");
                return None;
            }
        }
    }
    Some(myargs)
}
