use crossterm::terminal::size;
use regex::Regex;
use std::process;

// Gets the current terminal size
pub fn termsize() -> (usize, usize) {
    let mut session_width: usize = 0;
    let mut session_height: usize = 0;
    if let Ok((w, h)) = size() {
        session_width = w as usize;
        session_height = h as usize;
    } else {
        eprintln!(
            "Unable to get terminal size {} {}",
            session_width, session_height
        );
        process::exit(1);
    }
    (session_width, session_height)
}

// Shortens a package name for more aesthetic display to user
// e.g sys-cluster/kube-scheduler-1.29.1::gentoo to sys-cluster/kube-scheduler
pub fn shorten(packagename: &str) -> String {
    let regularexpression = Regex::new(r"(.*)-[0-9].*").unwrap();
    if let Some(capture) = regularexpression.captures(packagename) {
        capture[1].to_string()
    } else {
        packagename.to_string()
    }
}

// Calculates the longest length of shortened package names in a vector of absolute package names
pub fn longest(vec_of_strings: &Vec<&str>) -> u16 {
    let mut longest_length = 0;
    let mut _thislen = 0;
    for string_to_consider in vec_of_strings {
        let shortened_string = shorten(string_to_consider);
        _thislen = shortened_string.len() as u16;
        if _thislen > longest_length {
            longest_length = _thislen;
        }
    }
    longest_length
}

// Pretty prints a list of packages
pub fn package_list(plist: &Vec<&str>) {
    println!();
    let spaces: u16 = 4;
    let max_length = longest(plist);
    let (width, _height) = termsize();
    let width = width as u16;
    let number_of_items_per_line = width / (max_length + spaces);
    let mut counter = 0;
    for item in plist {
        let shortitem = shorten(item);
        print!("{shortitem}    ");
        counter += 1;
        if counter >= number_of_items_per_line {
            println!();
            counter = 0;
            continue;
        }
        for _filler in 0..=(max_length - (shortitem.len() as u16)) {
            print!(" ");
        }
    }
    if counter > 0 {
        println!();
    }
    println!();
}
