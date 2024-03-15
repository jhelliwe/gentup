use crate::Prompt::*;
use crossterm::style::{Color, SetForegroundColor};
use std::{io, process};

#[derive(PartialEq)]
pub enum Prompt {
    Review,
    PressCR,
}

// Prompt the user to continue, skip, quit etc
impl Prompt {
    pub fn user(userinput: &str, mode: Self) -> bool {
        if mode != PressCR {
            println!(
                "{} {}: Press return to continue, s to skip, q to quit",
                chevrons(Color::Green),
                userinput
            );
        } else {
            println!(
                "{} {}: Press return to continue, or q to quit",
                chevrons(Color::Green),
                userinput,
            );
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
            return false;
        }
        true
    }
}

pub fn chevrons(colour: Color) -> String {
    SetForegroundColor(colour).to_string() + ">>>" + &SetForegroundColor(Color::Grey).to_string()
}

pub fn revchevrons(colour: Color) -> String {
    SetForegroundColor(colour).to_string() + "<<<" + &SetForegroundColor(Color::Grey).to_string()
}
