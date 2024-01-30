use ansi_term::Colour;

pub fn three(colour: Colour) {
    print!("{}", colour.paint(">>> "));
}
