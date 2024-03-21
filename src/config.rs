// TODO - stop storing the users SMTP credentials in plaintext. Will encrypt
//
use crate::{linux::OsCall, mail, prompt, Prompt};
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
    pub clean_default: bool,
    pub trim_default: bool,
    pub email_address: String,
    pub mta_fqdn: String,
    pub auth: String,
    pub passwd: String,
}

// Implement a formatter for Config so we can display the contents
//
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "clean_default: {}\n\
            trim_default: {}\n\
            email_address: {}\n\
            mta_fqdn: {}\n\
            auth: {}\n\
            passwd: {}",
            self.clean_default,
            self.trim_default,
            self.email_address,
            self.mta_fqdn,
            self.auth,
            self.passwd,
        )
    }
}

impl Config {
    // Generate a default config
    //
    pub fn build_default() -> Self {
        Config {
            clean_default: false,
            trim_default: false,
            email_address: "root@localhost".to_string(),
            mta_fqdn: "localhost".to_string(),
            auth: "NONE".to_string(),
            passwd: "".to_string(),
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
        let _ = writeln!(config_file, "# Configuration options for gentup\n");
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
                    if let Some(switch) = getswitch("clean_default:", line) {
                        running_config.clean_default = switch;
                    }
                    if let Some(switch) = getswitch("trim_default:", line) {
                        running_config.trim_default = switch;
                    }
                    if let Some(param) = getparam("email_address:", line) {
                        running_config.email_address = param;
                    }
                    if let Some(param) = getparam("mta_fqdn:", line) {
                        running_config.mta_fqdn = param;
                    }
                    if let Some(param) = getparam("auth:", line) {
                        running_config.auth = param;
                    }
                    if let Some(param) = getparam("passwd:", line) {
                        running_config.passwd = param;
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
    println!("{} Entering setup", prompt::chevrons(Color::Green));
    //
    // Load or create the configuration file
    //
    let running_config: Config = if !Path::new(&CONFIG_FILE_PATH).exists() {
        println!(
            "{} Creating new configuration file",
            prompt::revchevrons(Color::Yellow)
        );
        Config::build_default().save()
    } else {
        println!(
            "{} Reading existing configuration file ({})",
            prompt::revchevrons(Color::Green),
            CONFIG_FILE_PATH,
        );
        Config::load()
    };
    //
    // Display the running configuration
    //
    println!(
        "{} The running configuration contains :\n\n{}\n",
        prompt::revchevrons(Color::Green),
        running_config
    );
    //
    // Edit the config
    //
    let optans = Prompt::Options.askuser("Edit the configuration? [y/n/q] ");
    if let Some(answer) = optans {
        if answer.eq("y\n") {
            let _ = OsCall::Interactive
                .execute(&["vi ", &CONFIG_FILE_PATH].concat(), "Launching editor");
        }
    }
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
    //
    // Edit the list of optional packages
    //
    let optans = Prompt::Options.askuser("Edit the list of optional packages to install? [y/n/q] ");
    if let Some(answer) = optans {
        if answer.eq("y\n") {
            let _ = OsCall::Interactive
                .execute(&["vi ", &PACKAGE_FILE_PATH].concat(), "Launching editor");
        }
    }
    //
    // Setup email
    //
    let optans = Prompt::Options.askuser("Setup email? [y/n/q] ");
    if let Some(answer) = optans {
        if answer.eq("y\n") {
            mail::setup();
        }
    }
    println!("{} Setup complete", prompt::chevrons(Color::Green));
}
