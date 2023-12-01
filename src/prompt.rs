use std::io;
use std::process;
use clearscreen;
use crate::PromptType::*;

pub fn ask_user(userinput: &str, mode: crate::PromptType) -> bool {
    if mode == ClearScreen {
        clearscreen::clear().expect("Terminfo problem. Cannot continue");
    }
    if mode != PressCR {
        println!("{}: Press return to continue, s to skip, q to quit", userinput);
    }
    else
    {
        println!("{}: Press return to continue, or q to quit", userinput);
    }

    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    if user_input.eq("q\n") { println!("Quitting at user request"); process::exit(0); }
    if user_input.eq("s\n") { println!("Skipping at user request"); return false; }
    println!("Acknowledged");
    true
}

