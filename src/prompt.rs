use std::io;
use std::process;
use clearscreen;

pub fn ask_user(userinput: &str) -> bool {
    clearscreen::clear().expect("Terminfo problem. Cannot continue");
    println!("{}: Press return to continue, s to skip, q to quit", userinput);
    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    if user_input.eq("q\n") { println!("Quitting at user request"); process::exit(0); }
    if user_input.eq("s\n") { println!("Skipping at user request"); return false; }
    println!("Acknowledged");
    true
    }

pub fn review_output(userinput: &str) -> bool {
    println!("{}: Press return to continue, s to skip, q to quit", userinput);
    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    if user_input.eq("q\n") { println!("Quitting at user request"); process::exit(0); }
    if user_input.eq("s\n") { println!("Skipping at user request"); return false; }
    println!("Acknowledged");
    true
    }

pub fn press_cr(userinput: &str) {
    println!("{}: Press return to continue, q to quit", userinput);
    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    if user_input.eq("q\n") { println!("Quitting at user request"); process::exit(0); }
    println!("Acknowledged");
    }
