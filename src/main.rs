// Gentoo Updater version 0.01a

use std::fs::{self,File};
use std::io::{self,BufRead,BufReader};
use std::process::Command;
use filetime::FileTime;
use chrono;


fn check_distro(required_distro: String) -> Result<String, String>  {
    let os_release = File::open("/etc/os-release").expect("/etc/os-release should be readable!");
    let readbuf = BufReader::new(os_release);
    let firstline = readbuf.lines().next().expect("Could not read /etc/os-release").unwrap();
    let parts = firstline.split('=');
    let parts: Vec<&str> = parts.collect();
    let detected_distro = parts[1].to_string();
    match required_distro.eq(&detected_distro) {
        true => Ok(detected_distro),
        false => Err(detected_distro),
    }
}

fn do_eix_sync() {
    let portage_metadata = fs::metadata("/var/db/repos/gentoo/metadata/timestamp").unwrap();
    let filestamp = FileTime::from_last_modification_time(&portage_metadata).seconds() ;
    let nowutc = chrono::offset::Utc::now();
    let nowstamp = nowutc.timestamp();

    if nowstamp - filestamp < (24 * 60 * 60) {
        println!("Last emerge --sync was less than 24 hours ago. We will skip doing an eix-sync");
    } else {
        // At this point we can do an eix-sync
    
        check_with_user("Step 1 is to update the Portage tree");
        let process = Command::new("eix-sync")
        //.arg("-p")
        //.arg("--depclean")
        .spawn()
        .expect("DEPCLEAN ERROR");
        let _output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
        };
        check_with_user("Please verify the output of eix-sync above");
   };
}

fn check_with_user(message: &str) {
    println!("{message}: Press Return to continue, Enter 'q' to to quit, or Ctrl-C to quit");
    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    if user_input.eq("q\n") { panic!("Quitting at user request"); }
}

fn main() {

    /* Check our running environment
     * Are we running on Gentoo? 
     */

    let distro = check_distro("Gentoo".to_string()).expect("This updater only works on Gentoo!");
    println!("Welcome to the {} Updater", distro);

    /* Now check the timestamp of the Gentoo package repo to prevent more than one sync per day
     * and if we are not too recent from the last emerge --sync, call eix-sync
     */

    do_eix_sync();
    
    println!("All done!!!");
}
