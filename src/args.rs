// Parse command line arguments

use crate::VERSION;
use std::env::{self, Args};

#[derive(Debug)]
pub struct GentupArgs {
    pub cleanup: bool,
    pub force: bool,
    pub background_fetch: bool,
    pub optional: bool,
}

impl GentupArgs {
    pub fn parse(args: Args) -> Result<Self, String> {
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
        let mut myargs = GentupArgs {
            cleanup: false,
            force: false,
            background_fetch: false,
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
                    -b  --background Perform source fetching in the background during update\n\
                    -c, --cleanup    Perform cleanup tasks only\n\
                    -f, --force      force eix-sync, bypassing the timestamp check\n\
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
                "-b" | "--background" => {
                    myargs.background_fetch = true;
                }
                "-c" | "--cleanup" => {
                    myargs.cleanup = true;
                }
                "-o" | "--optional" => {
                    myargs.optional = true;
                }
                _ => {
                    return Err(
                        "Error: usage - gentup [--help|--force|--background|--cleanup|--version]"
                            .to_string(),
                    );
                }
            }
        }
        Ok(myargs)
    }
}
