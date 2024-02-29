use crate::{
    prompt,
    CmdVerbose::{self, *},
    ShellOutResult,
};
use crossterm::{
    style::{Color, SetForegroundColor},
    terminal::size,
};
use execute::Execute;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
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
pub fn system_command(command_line: &str, status: &str, verbose: CmdVerbose) -> ShellOutResult {
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
            Interactive => {
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
pub fn exit_on_failure(shellout_result: &ShellOutResult) {
    match shellout_result {
        (Ok(_), status) => {
            if *status != 0 {
                eprintln!(
                    "{} The command had a non zero exit status. Please check.\n",
                    prompt::revchevrons(Color::Red)
                );
                process::exit(1);
            }
        }
        (Err(errors), _) => {
            eprintln!(
                "{} There was a problem executing the command: {}",
                prompt::revchevrons(Color::Red),
                errors
            );
            process::exit(1);
        }
    }
}

// Returns the name of the Linux distro we are running on. I don't actually check this IS Linux,
// because there is only me using it, and I'm not likely to run this on a Windows/Mac/FreeBSD box etc
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
            " but this updater only works on Gentoo Linux",
        ]
        .concat()),
    }
}

// This function returns the device name that the root filesystem resides on
pub fn getdev_rootfs() -> String {
    let mut rootfsdev = "None".to_string();
    let procmounts = fs::read_to_string("/proc/mounts");
    match procmounts {
        Ok(contents) => {
            for eachline in contents.lines() {
                if eachline.contains(" / ") {
                    let rootfsvec: Vec<&str> = eachline.split_whitespace().collect();
                    rootfsdev = rootfsvec[0].to_string();
                    break;
                }
            }
            rootfsdev.to_string()
        }
        Err(error) => {
            eprintln!("Error {}", error);
            "None".to_string()
        }
    }
}

// This function removed numeric elements of a string
pub fn stripchar(devicename: String) -> String {
    return devicename.chars().filter(|c| c.is_numeric()).collect();
}

// This function returns the major device number of a named device node
pub fn major_device_number(devnode: String) -> String {
    let shellout_result = system_command(&["ls -l ", &devnode].concat(), "", Quiet);
    exit_on_failure(&shellout_result);
    if let (Ok(output), _) = shellout_result {
        let lsvec: Vec<&str> = output.split_whitespace().collect();
        let maj = lsvec[4];
        let newmaj = stripchar(maj.to_string());
        return newmaj;
    }
    "0".to_string()
}

// This function returns the pathname of the rotational attribute of a named block device by major
// device number
pub fn syspath(major: String) -> String {
    ["/sys/dev/block/", &major, ":0/queue/rotational"].concat()
}

// This function returns a 1 if the root filesystem resides on a rotational device like a HDD or 0
// if the root filesystem resides on an SSD or thinly provisioned backing store
pub fn is_rotational() -> i32 {
    let return_value: i32 = 1;
    let device_name = getdev_rootfs();
    let device_major = major_device_number(device_name);
    let sys = syspath(device_major);
    let result = fs::read_to_string(sys);
    if let Ok(hdd) = result {
        return hdd.trim().parse::<i32>().unwrap();
    }
    return_value
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
