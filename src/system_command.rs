use execute::Execute;
use std::process::{Command};

pub fn system_command(command_line: &str) {
    println!("Executing {}", command_line);
    let mut command_words=Vec::new();
    for word in command_line.split_whitespace() {
        command_words.push(word);
    }
    let mut command = Command::new(command_words[0]);
    for argument in command_words.iter().skip(1) {
        command.arg(argument);
    }
    let output = command.execute_output().unwrap();
    if let Some(exit_code) = output.status.code() {
        if exit_code == 0 {
            println!("Ok.");
        } else {
            eprintln!("Failed.");
        }
    } else {
        eprintln!("Interrupted!");
    }
}

pub fn call_fstrim() {
    system_command("fstrim -a -v");
}
