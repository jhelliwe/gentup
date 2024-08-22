use crate::{
    linux::{self, OsCall},
    mail, prompt, Prompt,
};
use crossterm::style::Color;
use std::{
    fmt,
    fs::{self, File},
    io::Write,
    path::Path,
    process,
};

pub static CONFIG_FILE_PATH: &str = "/etc/conf.d/gentup";
pub static PACKAGE_FILE_PATH: &str = "/etc/default/gentup";

// Define a struct to hold the configuration options
//
pub struct Config {
    pub cleanup_default: bool,
    pub trim_default: bool,
    pub background_default: bool,
    pub email_address: String,
}

// Implement a formatter for Config so we can display the contents
//
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "cleanup_default: {}\n\
            trim_default: {}\n\
            background_default: {}\n\
            email_address: {}\n",
            self.cleanup_default, self.trim_default, self.background_default, self.email_address,
        )
    }
}

impl Config {
    // Generate a default config
    //
    pub fn build_default() -> Self {
        Config {
            cleanup_default: false,
            trim_default: false,
            background_default: false,
            email_address: "root@localhost".to_string(),
        }
    }

    // Save the running config out to the config file
    //
    pub fn save(self) -> Self {
        let path = Path::new(&CONFIG_FILE_PATH);
        let display = path.display();
        let mut config_file = match File::create(path) {
            Err(error) => {
                eprintln!("Could not create {} - {}", display, error);
                process::exit(1);
            }
            Ok(config_file) => config_file,
        };
        let _ = writeln!(
            config_file,
            "# Configuration options for gentup\n\
            # post-update cleanup, true or false\n\
            # post-update trim, true or false\n\
            # background package downloads, true or false\n\
            # email address to send update reports to\n\
            "
        );
        let _ = writeln!(config_file, "{}", self);
        self
    }

    // Load the config file into the running config
    //
    pub fn load() -> Self {
        let getswitch = move |p, l: &str| -> Option<bool> {
            let mut c = None;
            let value = l.replace(p, "").to_string();
            let trimmed = value.trim();
            if l.contains(p) {
                match trimmed {
                    "true" => c = Some(true),
                    "false" => c = Some(false),
                    _ => {
                        println!(
                            "{} Syntax error in the config file: {}",
                            prompt::revchevrons(Color::Red),
                            l
                        );
                        c = None;
                    }
                }
            }
            c
        };
        let getparam = move |p, l: &str| -> Option<String> {
            let mut _c = None;
            let value = l.replace(p, "").to_string();
            let trimmed = value.trim();
            if l.contains(p) {
                _c = Some(trimmed.to_string())
            } else {
                _c = None
            }
            _c
        };
        let mut running_config = Config::build_default();
        let fileopt = fs::read_to_string(CONFIG_FILE_PATH);
        match fileopt {
            Ok(contents) => {
                for line in contents.lines() {
                    if let Some(switch) = getswitch("cleanup_default:", line) {
                        running_config.cleanup_default = switch;
                    }
                    if let Some(switch) = getswitch("trim_default:", line) {
                        running_config.trim_default = switch;
                    }
                    if let Some(switch) = getswitch("background_default:", line) {
                        running_config.background_default = switch;
                    }
                    if let Some(param) = getparam("email_address:", line) {
                        running_config.email_address = param;
                    }
                }
            }
            Err(error) => {
                println!(
                    "{} Could not read {} - {}",
                    prompt::revchevrons(Color::Red),
                    CONFIG_FILE_PATH,
                    error
                );
                process::exit(1);
            }
        }
        running_config
    }
}

// Interactive setup
//
pub fn setup() {
    loop {
        //
        // Load or create the configuration file
        //
        let mut running_config: Config = if !Path::new(&CONFIG_FILE_PATH).exists() {
            Config::build_default().save()
        } else {
            Config::load()
        };

        //
        // Display the running configuration
        //

        println!(
            "{} The running configuration contains :\n\n{}",
            prompt::revchevrons(Color::Green),
            running_config
        );

        //
        // Display the list of optional packages
        //

        let optlist = fs::read_to_string(PACKAGE_FILE_PATH);
        if let Ok(plist) = optlist {
            println!(
                "{} Optional package list contains\n\n{}",
                prompt::revchevrons(Color::Green),
                plist
            );
        }

        let optans = Prompt::Options.askuser("Select c to edit the configuration, p to edit the package list, t to send a test email, or q to quit [c|p|t|q]");

        if let Some(answer) = optans {
            if answer.eq("c\n") {
                let _ = OsCall::Interactive
                    .execute(&["vi ", CONFIG_FILE_PATH].concat(), "Launching editor");
                running_config = Config::load();
            }
            if answer.eq("p\n") {
                let _ = OsCall::Interactive
                    .execute(&["vi ", PACKAGE_FILE_PATH].concat(), "Launching editor");
            }
            if answer.eq("t\n") {
                mail::test_mail(&running_config);
                linux::clearscreen();
                println!("{} Test email sent", prompt::revchevrons(Color::Green));
                continue;
            }
        }
        linux::clearscreen();
    }
}
