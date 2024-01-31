use crate::chevrons;
use crate::linux;
use crate::prompt::*;
use crate::tabulate;
use crate::PromptType;
use crate::Upgrade;
use ansi_term::Colour;
use filetime::FileTime;
use std::fs;
use std::io::Write;
use std::process;

pub fn eix_diff() -> bool {
    let shellout_result = linux::system_command_quiet("eix-diff");
    linux::exit_on_failure(&shellout_result);
    match shellout_result {
        (Ok(output), _) => {
            let mut pending_updates = Vec::new();
            for line in output.split('\n') {
                if line.starts_with("[U") {
                    let mut words = line.split_whitespace();
                    let mut word: Option<&str> = Some("");
                    for _counter in 1..=3 {
                        word = words.next();
                    }
                    match word {
                        Some(word) => {
                            pending_updates.push(word);
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
            let num_updates = pending_updates.len();
            match num_updates {
                0 => {
                    chevrons::three(Colour::Yellow);
                    println!("There are no pending updates");
                    return false;
                }
                1 => {
                    chevrons::three(Colour::Green);
                    println!("There is 1 package pending an update");
                }
                _ => {
                    println!();
                    chevrons::three(Colour::Green);
                    println!("There are {} packages pending updates", num_updates);
                }
            }
            tabulate::package_list(&pending_updates);
            true
        }
        (Err(_), _) => {
            chevrons::three(Colour::Red);
            eprintln!("Error calling eix-diff");
            false
        }
    }
}

// This function checks if the last portage sync was too recent (<=24 hours ago)
//
pub fn too_recent() -> bool {
    let portage_metadata = fs::metadata("/var/db/repos/gentoo/metadata/timestamp").unwrap();
    let filestamp = FileTime::from_last_modification_time(&portage_metadata).seconds();
    let nowutc = chrono::offset::Utc::now();
    let nowstamp = nowutc.timestamp();
    if nowstamp - filestamp < (24 * 60 * 60) {
        chevrons::three(Colour::Yellow);
        println!("Last sync was too recent: Skipping sync phase");
        true
    } else {
        false
    }
}

// This function checks that a named package is installed.
//
pub fn package_is_missing(package: &str) -> bool {
    print!(".");
    std::io::stdout().flush().unwrap();
    let shellout_result = linux::system_command_quiet(&["equery l ", package].concat());
    match shellout_result {
        (Ok(_), return_code) => {
            if return_code != 0 {
                println!();
                chevrons::three(Colour::Yellow);
                println!("{} is not installed", package);
                return true;
            }
            false
        }
        (Err(returned_error), _) => {
            eprintln!();
            chevrons::three(Colour::Red);
            eprintln!("Problem running command: {}", returned_error);
            process::exit(1);
        }
    }
}

// This function updates the package tree metadata for Gentoo Linux
//
pub fn do_eix_sync() {
    chevrons::three(Colour::Green);
    println!("Downloading latest package tree - please wait");
    let shellout_result = linux::system_command_quiet("eix-sync -q");
    linux::exit_on_failure(&shellout_result);
}

// This function calls eix to check if the named package is due an upgrade
//
pub fn package_outdated(package: &str) -> bool {
    let shellout_result = linux::system_command_quiet(&["eix -u ", package].concat());
    match shellout_result {
        (Ok(_), return_status) => {
            if return_status != 0 {
                return false;
            }
            chevrons::three(Colour::Yellow);
            println!("{} needs upgrade", package);
            true
        }
        (Err(_), returned_error) => {
            chevrons::three(Colour::Red);
            eprintln!("Command returned {}", returned_error);
            process::exit(1);
        }
    }
}

// This function performs an update of the named package
//
pub fn upgrade_package(package: &str) {
    let shellout_result = linux::system_command(&["emerge --quiet -1av ", package].concat());
    linux::exit_on_failure(&shellout_result);
}

// This function performs an update of the world set - i.e a full system upgrade
//
pub fn upgrade_world(run_type: Upgrade) {
    match run_type {
        Upgrade::Real => {
            let shellout_result = linux::system_command(
                "emerge --quiet -uNDv --with-bdeps y --changed-use --complete-graph @world",
            );
            linux::exit_on_failure(&shellout_result);
        }
        Upgrade::Fetch => {
            let shellout_result = linux::system_command(
                "emerge --fetchonly -uNDv --with-bdeps y --changed-use --complete-graph @world",
            );
            linux::exit_on_failure(&shellout_result);
        }
        _ => {} // Future expansion
    }
}

pub fn elogv() {
    let _shellout_result = linux::system_command("elogv");
}

// This function does a depclean
//
pub fn depclean(run_type: Upgrade) -> i32 {
    match run_type {
        Upgrade::Pretend => {
            chevrons::three(Colour::Green);
            println!("Performing dependency check... Please wait");
            let shellout_result = linux::system_command_quiet("emerge -p --depclean");
            linux::exit_on_failure(&shellout_result);
            if let (Ok(output), _) = shellout_result {
                let lines = output.split('\n');
                for line in lines {
                    println!("{line}");
                    if line.starts_with("Number to remove") {
                        let mut words = line.split_whitespace();
                        let mut word: Option<&str> = Some("");
                        for _counter in 1..=4 {
                            word = words.next();
                        }
                        match word {
                            Some(word) => {
                                return word.parse().unwrap();
                            }
                            None => {
                                return 0;
                            }
                        }
                    }
                }
            }

            0
        }

        Upgrade::Real => {
            let shellout_result = linux::system_command("emerge --depclean");
            linux::exit_on_failure(&shellout_result);
            ask_user(
                "Please verify the output of emerge --depclean above",
                PromptType::PressCR,
            );
            0
        }
        _ => 0,
    }
}

pub fn revdep_rebuild(run_type: Upgrade) -> bool {
    match run_type {
        Upgrade::Pretend => {
            chevrons::three(Colour::Green);
            println!("Performing reverse dependency check... Please wait");
            let shellout_result = linux::system_command_quiet("revdep-rebuild -ip");
            linux::exit_on_failure(&shellout_result);
            if let (Ok(output), _) = shellout_result {
                let lines = output.split('\n');
                for line in lines {
                    println!("{line}");
                    if line.starts_with("Your systen is consistent") {
                        return true;
                    }
                }
            }
            false
        }
        Upgrade::Real => {
            let shellout_result = linux::system_command("revdep-rebuild");
            linux::exit_on_failure(&shellout_result);
            ask_user(
                "Please verify the output of revdep-rebuild above",
                PromptType::PressCR,
            );
            true
        }
        _ => false,
    }
}

// This function calls the portage sanity checker
pub fn eix_test_obsolete() {
    chevrons::three(Colour::Green);
    println!("Performing portage hygiene tests");
    let shellout_result = linux::system_command("eix-test-obsolete");
    linux::exit_on_failure(&shellout_result);
}

// This function cleans up old kernels
pub fn eclean_kernel() {
    let shellout_result = linux::system_command("eclean-kernel -Aa");
    linux::exit_on_failure(&shellout_result);
}

// This function removes old unused package tarballs
//
pub fn eclean_distfiles() {
    let shellout_result = linux::system_command("eclean -d distfiles");
    linux::exit_on_failure(&shellout_result);
}

pub fn eix_update() {
    let shellout_result = linux::system_command_quiet("eix-update");
    linux::exit_on_failure(&shellout_result);
}
