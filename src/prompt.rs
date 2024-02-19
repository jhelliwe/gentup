use crate::{chevrons, PromptType::*};
use crossterm::{
    cursor, execute,
    style::Color,
    terminal::{self, ClearType},
};
use std::{io, process};

// Prompt the user to continue, skip, quit etc
pub fn ask_user(userinput: &str, mode: crate::PromptType) -> bool {
    if mode == ClearScreen {
        let _ignore = execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        );
    }
    if mode != PressCR {
        println!(
            "{} {}: Press return to continue, s to skip, q to quit",
            chevrons::three(Color::Green),
            userinput
        );
    } else {
        println!(
            "{} {}: Press return to continue, or q to quit",
            chevrons::three(Color::Green),
            userinput,
        );
    }

    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    if user_input.eq("q\n") {
        println!("{} Quitting at user request", chevrons::three(Color::Green));
        process::exit(0);
    }
    if user_input.eq("s\n") {
        println!("{} Skipping at user request", chevrons::three(Color::Green));
        return false;
    }
    true
}
