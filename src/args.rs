// Parse command line arguments

use crate::{GentupArgs, VERSION};
use std::env::{self, Args};

pub fn parsecmdlinargs(args: Args) -> Result<GentupArgs, String> {
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
    // Parse command line arguments
    let mut myargs = GentupArgs {
        cleanup: false,
        force: false,
        separate: false,
        optional: false,
    };

    let mut first = true;
    for arg in args {
        if first {
            first = false;
            continue;
        }
        match &arg[..] {
            "-h" | "--help" => {
                return Err("Usage:\n\n \
                    gentup [options]\n \
                    Options:\n\n\
                    -c, --cleanup    Perform cleanup tasks only\n\
                    -f, --force      force eix-sync, bypassing the timestamp check\n\
                    -s  --separate   Perform source fetching separately before update\n\
                    -o, --optional   Install optional packages from /etc/default/gentup\n\
                    -h, --help       Display this help text, then exit\n\
                    -V, --version    Display the program version\
                "
                .to_string());
            }
            "-V" | "--version" => {
                return Err(format!("gentup version {}", VERSION));
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
            "-o" | "--optional" => {
                myargs.optional = true;
            }
            _ => {
                return Err(
                    "Error: usage - gentup [--help|--force|--separate|--cleanup|--version]"
                        .to_string(),
                );
            }
        }
    }
    Ok(myargs)
}
