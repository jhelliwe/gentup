use crate::prompt;
use crossterm::{
    style::{Color, SetForegroundColor},
    terminal::size,
};
use execute::Execute;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    process::{self, Command, Stdio},
};
use terminal_spinners::{SpinnerBuilder, LINE};

// Define a new type, OsCall which executes an external OS command
pub enum OsCall {
    Interactive, // stdin, stdout and stderr are left attached to the tty allowing the user to interact
    Spinner,     // stdout is piped allowing OsCall to capture the stdout and return it as a String
    Quiet, // stdout and stderr are piped allowing OsCall to capture them and return them in a String
}
pub type ShellOutResult = Result<(String, i32), Box<dyn Error>>; // ShellOutResult is returned from an OsCall
pub trait CouldFail {
    // OsCalls could fail, and the failures need to be handled
    fn exit_if_failed(self) -> ShellOutResult;
}
impl CouldFail for ShellOutResult {
    // Generic handler for failed OsCalls
    fn exit_if_failed(self) -> ShellOutResult {
        match self {
            Ok((_, status)) => {
                if status != 0 {
                    eprintln!(
                        "{} The command had a non zero exit status. Please check.\n",
                        prompt::revchevrons(Color::Red)
                    );
                    process::exit(1);
                }
            }
            Err(errors) => {
                eprintln!(
                    "{} There was a problem executing the command: {}",
                    prompt::revchevrons(Color::Red),
                    errors
                );
                process::exit(1);
            }
        }
        self
    }
}
impl OsCall {
    // Fork and exec an external command. Waits for completion
    pub fn execute(self, command_line: &str, status: &str) -> ShellOutResult {
        let mut command_words = Vec::new();
        for word in command_line.split_whitespace() {
            command_words.push(word);
        }
        let mut command = Command::new(command_words[0]);
        for argument in command_words.iter().skip(1) {
            command.arg(argument);
        }
        let results = {
            match self {
                // Spinner - executes a command via the OS with a progress spinner, returns
                // stdout to the calling function
                OsCall::Spinner => {
                    command.stdout(Stdio::piped());
                    let text = prompt::chevrons(Color::Green)
                        + " "
                        + status
                        + ": "
                        + &SetForegroundColor(Color::Cyan).to_string()
                        + command_line
                        + &SetForegroundColor(Color::Grey).to_string()
                        + " ";
                    let handle = SpinnerBuilder::new()
                        .spinner(&LINE)
                        .prefix(text)
                        .text(" ")
                        .start();
                    let result = command.execute_output();
                    handle.done();
                    result
                }
                // Interactive - executes a command via the OS leaving stdin and stdout attached to
                // the tty
                OsCall::Interactive => {
                    println!(
                        "{} {}: {}{}{}",
                        prompt::chevrons(Color::Green),
                        status,
                        &SetForegroundColor(Color::Cyan),
                        command_line,
                        &SetForegroundColor(Color::Grey)
                    );
                    command.execute_output()
                }
                // Quiet - executes a command via the OS returning stdout and stderr to the calling
                // function
                OsCall::Quiet => {
                    command.stdout(Stdio::piped());
                    command.stderr(Stdio::piped());
                    command.execute_output()
                }
            }
        };
        match results {
            Ok(output) => Ok((
                (String::from_utf8_lossy(&output.stdout).to_string()),
                output.status.code().unwrap(),
            )),
            Err(errors) => Err(Box::new(errors)),
        }
    }
}

pub fn call_fstrim() {
    let _ = OsCall::Spinner
        .execute("fstrim -a", "Trimming filesystems")
        .exit_if_failed();
}

// Returns the name of the Linux distro we are running on.
pub fn check_distro(required_distro: &str) -> Result<String, String> {
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
    match required_distro.eq(&detected_distro) {
        true => Ok(detected_distro),
        false => Err([
            "Detected this system is running ",
            &detected_distro,
            " but this updater only works on ",
            &required_distro,
            " Linux",
        ]
        .concat()),
    }
}

// This function removed numeric elements of a string
pub fn stripchar(devicename: String) -> String {
    return devicename.chars().filter(|c| c.is_numeric()).collect();
}

// Gets the current terminal size
pub fn termsize() -> (usize, usize) {
    let mut session_width: usize = 0;
    let mut session_height: usize = 0;
    if let Ok((w, h)) = size() {
        session_width = w as usize;
        session_height = h as usize;
    } else {
        eprintln!(
            "Unable to get terminal size {} {}",
            session_width, session_height
        );
        process::exit(1);
    }
    (session_width, session_height)
}

// Returns the running kernel version
pub fn running_kernel() -> String {
    if let Ok((output, _)) = OsCall::Quiet.execute("uname -r", "") {
        stripchar(output)
    } else {
        String::new()
    }
}
