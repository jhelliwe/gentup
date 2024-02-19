use crossterm::style::{Color, SetForegroundColor};

pub fn three(colour: Color) -> String {
    SetForegroundColor(colour).to_string() + ">>>" + &SetForegroundColor(Color::Grey).to_string()
}

pub fn eerht(colour: Color) -> String {
    SetForegroundColor(colour).to_string() + "<<<" + &SetForegroundColor(Color::Grey).to_string()
}
