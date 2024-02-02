use crate::{chevrons, PromptType::*};
use crossterm::{
    cursor, execute,
    style::Color,
    terminal::{self, ClearType},
};
use std::{
    io::{self, Write},
    process,
    time::Duration,
};

pub fn ask_user(userinput: &str, mode: crate::PromptType) -> bool {
    if mode == ClearScreen {
        let _ignore = execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        );
    }
    if mode != PressCR {
        chevrons::three(Color::Green);
        println!(
            "{}: Press return to continue, s to skip, q to quit",
            userinput
        );
    } else {
        chevrons::three(Color::Green);
        println!("{}: Press return to continue, or q to quit", userinput);
    }

    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    if user_input.eq("q\n") {
        chevrons::three(Color::Green);
        println!("Quitting at user request");
        process::exit(0);
    }
    if user_input.eq("s\n") {
        chevrons::three(Color::Green);
        println!("Skipping at user request");
        return false;
    }
    true
}

pub fn dots(manydots: i32) {
    for _counter in 0..=manydots {
        print!(".");
        std::thread::sleep(Duration::from_millis(1000));
        std::io::stdout().flush().unwrap();
    }
    println!();
}
