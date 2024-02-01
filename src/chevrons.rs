use crossterm::{
    execute,
    style::{Color, SetForegroundColor},
};
use std::io;

pub fn three(colour: Color) {
    //print!("{}", colour.paint(">>> "));
    let _ignore = execute!(io::stdout(), SetForegroundColor(colour));
    print!(">>> ");
    let _ignore = execute!(io::stdout(), SetForegroundColor(Color::Grey));
}
