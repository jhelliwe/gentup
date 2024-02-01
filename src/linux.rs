use crate::{chevrons, prompt, PromptType};
use crossterm::{
    cursor, execute,
    style::Color,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use execute::Execute;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    process::{self, Command},
    time::Duration,
};

pub fn system_command(command_line: &str) -> (Result<String, Box<dyn Error>>, i32) {
    std::thread::sleep(Duration::from_millis(2000));
    let _ignore = execute!(io::stdout(), EnterAlternateScreen);
    let _ignore = execute!(
        io::stdout(),
        cursor::SavePosition,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    );
    let mut command_words = Vec::new();
    for word in command_line.split_whitespace() {
        command_words.push(word);
    }
    let mut command = Command::new(command_words[0]);
    for argument in command_words.iter().skip(1) {
        command.arg(argument);
    }
    chevrons::three(Color::Green);
    println!("Working... ({})", command_line);
    let results = command.execute_output();
    prompt::ask_user("Please review above output?\t\t", PromptType::PressCR);
    let _ignore = execute!(io::stdout(), LeaveAlternateScreen, cursor::RestorePosition);
    match results {
        Ok(output) => (
            Ok(String::from_utf8_lossy(&output.stdout).to_string()),
            output.status.code().unwrap(),
        ),
        Err(errors) => (Err(Box::new(errors)), 1),
    }
}

pub fn system_command_simple(command_line: &str) -> (Result<String, Box<dyn Error>>, i32) {
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
    let shellout_results = system_command_simple("fstrim -a -v");
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
    chevrons::three(Color::Green);
    println!("Running on {}: OK", detected_distro);
    match required_distro.eq(&detected_distro) {
        true => Ok(detected_distro),
        false => Err(detected_distro),
    }
}
