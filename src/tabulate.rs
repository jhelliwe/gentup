use crossterm::terminal::size;
use std::process;

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

pub fn package_list(plist: &Vec<&str>) {
    println!();
    let spaces: u16 = 4;
    let m = longest(plist);
    let (width, _height) = termsize();
    let w = width as u16;
    let n = w / (m + spaces);
    let mut counter = 0;
    for item in plist {
        print!("{item}    ");
        counter += 1;
        if counter >= n {
            println!();
            counter = 0;
            continue;
        }
        for _filler in 0..=(m - (item.len() as u16)) {
            print!(" ");
        }
    }
    println!();
    println!();
}

pub fn longest(thelist: &Vec<&str>) -> u16 {
    let mut l = 0;
    let mut _thislen = 0;
    for stc in thelist {
        _thislen = stc.len() as u16;
        if _thislen > l {
            l = _thislen;
        }
    }
    l
}
