use crate::{
    chevrons,
    CmdVerbose::{self, *},
};
use crossterm::{
    execute,
    style::{Color, SetForegroundColor},
};
use execute::Execute;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    process::{self, Command, Stdio},
};
use terminal_spinners::{SpinnerBuilder, LINE};

// This function executes "command_line" with an optional progress spinner, and returns the stdout as a String to the
// caller of the function
//
// A CmdVerbose type of Interactive leaves stdin and stdout attached to the terminal session.
// Due to the fact that stdout is left alone, we cannot capture stdout to the String, so this
// function will return an empty string to the caller, for Interactive shellouts.
//
// A CmdVerbose type of NonInteractive produces no output, provides a progress spinner and returns
// stdout captured as a String
//
// A CmdVerbose type of Quiet is ditto, but with no spinner. Returns stdout as a String
//
pub fn system_command(
    command_line: &str,
    status: &str,
    verbose: CmdVerbose,
) -> (Result<String, Box<dyn Error>>, i32) {
    let mut command_words = Vec::new();
    for word in command_line.split_whitespace() {
        command_words.push(word);
    }
    let mut command = Command::new(command_words[0]);
    for argument in command_words.iter().skip(1) {
        command.arg(argument);
    }
    let results = {
        match verbose {
            NonInteractive => {
                command.stdout(Stdio::piped());
                let text = chevrons::three(Color::Green)
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
            Interactive => {
                print!("{} {}: ", chevrons::three(Color::Green), status);
                let _ignore = execute!(io::stdout(), SetForegroundColor(Color::Cyan));
                println!("{}", command_line);
                let _ignore = execute!(io::stdout(), SetForegroundColor(Color::Grey));
                command.execute_output()
            }
            Quiet => {
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());
                command.execute_output()
            }
        }
    };
    match results {
        Ok(output) => (
            Ok(String::from_utf8_lossy(&output.stdout).to_string()),
            output.status.code().unwrap(),
        ),
        Err(errors) => (Err(Box::new(errors)), 1),
    }
}

pub fn call_fstrim() {
    let shellout_results = system_command("fstrim -a", "Trimming filesystems", NonInteractive);
    exit_on_failure(&shellout_results);
}

// This function takes the result from the last system command execution and exits if there was a
// failure running the previous command
pub fn exit_on_failure(shellout_result: &(Result<String, Box<dyn Error>>, i32)) {
    match shellout_result {
        (Ok(_), status) => {
            if *status != 0 {
                eprintln!(
                    "{} The command had a non zero exit status. Please check.\n",
                    chevrons::eerht(Color::Red)
                );
                process::exit(1);
            }
        }
        (Err(errors), _) => {
            eprintln!(
                "{} There was a problem executing the command: {}",
                chevrons::eerht(Color::Red),
                errors
            );
            process::exit(1);
        }
    }
}

// Returns the name of the Linux distro we are running on. I don't actually check this IS Linux,
// because there is only me using it, and I'm not likely to run this on a Windows/Mac/FreeBSD box etc
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
    match required_distro.eq(&detected_distro) {
        true => Ok(detected_distro),
        false => Err(detected_distro),
    }
}
