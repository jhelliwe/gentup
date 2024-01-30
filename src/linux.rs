use crate::chevrons;
use ansi_term::Colour;
use execute::Execute;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::{self, Command};

pub fn system_command(command_line: &str) -> (Result<String, Box<dyn Error>>, i32) {
    let mut command_words = Vec::new();
    for word in command_line.split_whitespace() {
        command_words.push(word);
    }
    let mut command = Command::new(command_words[0]);
    for argument in command_words.iter().skip(1) {
        command.arg(argument);
    }
    let results = command.execute_output();
    match results {
        Ok(output) => (
            Ok(String::from_utf8_lossy(&output.stdout).to_string()),
            output.status.code().unwrap(),
        ),
        Err(errors) => (Err(Box::new(errors)), 1),
    }
}

pub fn system_command_quiet(command_line: &str) -> (Result<String, Box<dyn Error>>, i32) {
    let mut command_words = Vec::new();
    for word in command_line.split_whitespace() {
        command_words.push(word);
    }
    let mut command = Command::new(command_words[0]);
    for argument in command_words.iter().skip(1) {
        command.arg(argument);
    }
    let results = command.output();
    match results {
        Ok(output) => (
            Ok(String::from_utf8_lossy(&output.stdout).to_string()),
            output.status.code().unwrap(),
        ),
        Err(errors) => (Err(Box::new(errors)), 1),
    }
}

pub fn call_fstrim() {
    let shellout_results = system_command("fstrim -a -v");
    exit_on_failure(&shellout_results);
}

pub fn exit_on_failure(shellout_result: &(Result<String, Box<dyn Error>>, i32)) {
    match shellout_result {
        (Ok(_), status) => {
            if *status != 0 {
                eprintln!("<<< The command had a non zero exit status. Please check.");
                process::exit(1);
            }
        }
        (Err(errors), _) => {
            eprintln!("<<< There was a problem executing the command: {}", errors);
            process::exit(1);
        }
    }
}

pub fn check_distro(required_distro: String) -> Result<String, String> {
    let os_release = File::open("/etc/os-release").expect("/etc/os-release should be readable!");
    let readbuf = BufReader::new(os_release);
    let firstline = readbuf
        .lines()
        .next()
        .expect("Could not read /etc/os-release")
        .unwrap();
    let parts = firstline.split('=');
    let parts: Vec<&str> = parts.collect();
    let detected_distro = parts[1].to_string();
    chevrons::three(Colour::Green);
    println!("Running on {}: OK", detected_distro);
    match required_distro.eq(&detected_distro) {
        true => Ok(detected_distro),
        false => Err(detected_distro),
    }
}
