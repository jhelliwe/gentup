use crate::Prompt::*;
use crossterm::style::{Color, SetForegroundColor};
use std::{
    io::{self, stdout, Write},
    process,
};

// Prompt the user to continue, skip, quit etc
#[derive(PartialEq)]
pub enum Prompt {
    AllowSkip,
    PressReturn,
    Options,
}
impl Prompt {
    pub fn askuser(self, prompt: &str) -> Option<String> {
        match self {
            AllowSkip => println!(
                "{} {}: Press return to continue, s to skip, q to quit",
                chevrons(Color::Green),
                prompt
            ),
            PressReturn => println!(
                "{} {}: Press return to continue, or q to quit",
                chevrons(Color::Green),
                prompt
            ),
            Options => {
                print!("{} {}: ", chevrons(Color::Green), prompt);
                let _ = stdout().flush();
            }
        }
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        if user_input.eq("q\n") {
            println!("{} Quitting at user request", chevrons(Color::Green));
            process::exit(0);
        }
        if user_input.eq("s\n") {
            println!("{} Skipping at user request", chevrons(Color::Green));
            return None;
        }
        Some(user_input)
    }
}

pub fn chevrons(colour: Color) -> String {
    SetForegroundColor(colour).to_string() + ">>>" + &SetForegroundColor(Color::Grey).to_string()
}

pub fn revchevrons(colour: Color) -> String {
    SetForegroundColor(colour).to_string() + "<<<" + &SetForegroundColor(Color::Grey).to_string()
}
