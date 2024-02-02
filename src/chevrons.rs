use crossterm::{
    execute,
    style::{Color, SetForegroundColor},
};
use std::io;

pub fn three(colour: Color) {
    let _ignore = execute!(io::stdout(), SetForegroundColor(colour));
    print!(">>> ");
    let _ignore = execute!(io::stdout(), SetForegroundColor(Color::Grey));
}

pub fn eerht(colour: Color) {
    let _ignore = execute!(io::stdout(), SetForegroundColor(colour));
    print!("<<< ");
    let _ignore = execute!(io::stdout(), SetForegroundColor(Color::Grey));
}
