use crate::{chevrons, linux, prompt::*, tabulate, PromptType, Upgrade};
use crossterm::style::Color;
use filetime::FileTime;
use std::{fs, io::Write, process};

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
                    chevrons::three(Color::Blue);
                    println!("There are no pending updates");
                    return false;
                }
                1 => {
                    chevrons::three(Color::Yellow);
                    println!("There is 1 package pending an update");
                }
                _ => {
                    chevrons::three(Color::Yellow);
                    println!("There are {} packages pending updates", num_updates);
                }
            }
            tabulate::package_list(&pending_updates);
            true
        }
        (Err(_), _) => {
            chevrons::three(Color::Red);
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
        chevrons::three(Color::Yellow);
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
                chevrons::three(Color::Yellow);
                println!("{} is not installed", package);
                return true;
            }
            false
        }
        (Err(returned_error), _) => {
            eprintln!();
            chevrons::three(Color::Red);
            eprintln!("Problem running command: {}", returned_error);
            process::exit(1);
        }
    }
}

// This function updates the package tree metadata for Gentoo Linux
//
pub fn do_eix_sync() {
    let shellout_result =
        linux::system_command_non_interactive(
            "eix-sync", 
            "Syncing package tree"
            );
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
            chevrons::three(Color::Yellow);
            println!("{} needs upgrade", package);
            true
        }
        (Err(_), returned_error) => {
            chevrons::three(Color::Red);
            eprintln!("Command returned {}", returned_error);
            process::exit(1);
        }
    }
}

// This function performs an update of the named package
//
pub fn upgrade_package(package: &str) {
    let shellout_result = linux::system_command_interactive(
        &["emerge --quiet -1av ", package].concat(),
        "Upgrading package",
    );
    linux::exit_on_failure(&shellout_result);
}

// This function performs an update of the world set - i.e a full system upgrade
//
pub fn upgrade_world(run_type: Upgrade) {
    match run_type {
        Upgrade::Real => {
            let shellout_result = linux::system_command_interactive(
                "emerge --quiet -uNDv --with-bdeps y --changed-use --complete-graph @world",
                "Updating world set",
            );
            linux::exit_on_failure(&shellout_result);
        }
        Upgrade::Fetch => {
            let shellout_result = linux::system_command_non_interactive(
                "emerge --quiet --fetchonly -uNDv --with-bdeps y --changed-use --complete-graph @world",
                "Fetching sources",
            );
            linux::exit_on_failure(&shellout_result);
        }
        _ => {} // Future expansion
    }
}

pub fn elogv() {
    let _shellout_result =
        linux::system_command_interactive(
            "elogv", 
            "Checking for new ebuild logs"
            );
}

// This function does a depclean
//
pub fn depclean(run_type: Upgrade) -> i32 {
    match run_type {
        Upgrade::Pretend => {
            let shellout_result = linux::system_command_non_interactive(
                "emerge -p --depclean",
                "Checking for orphaned dependencies",
            );
            linux::exit_on_failure(&shellout_result);
            if let (Ok(output), _) = shellout_result {
                let lines = output.split('\n');
                for line in lines {
                    if line.starts_with("Number to remove") {
                        let mut words = line.split_whitespace();
                        let mut word: Option<&str> = Some("");
                        for _counter in 0..=3 {
                            word = words.next();
                        }
                        match word {
                            Some(word) => {
                                let numdep = word.parse().unwrap();
                                if numdep == 0 {
                                    chevrons::eerht(Color::Blue);
                                } else {
                                    chevrons::eerht(Color::Yellow);
                                }
                                println!("Found {} dependencies to clean", numdep);
                                return numdep;
                            }
                            None => {
                                chevrons::eerht(Color::Green);
                                println!("There are no orphamed dependencies");
                                return 0;
                            }
                        }
                    }
                }
            }

            0
        }

        Upgrade::Real => {
            let shellout_result = linux::system_command_interactive(
                "emerge --depclean",
                "Removing orphaned dependencies",
            );
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
            let shellout_result = linux::system_command_non_interactive(
                "revdep-rebuild -ip",
                "Checking reverse dependencies",
            );
            linux::exit_on_failure(&shellout_result);
            if let (Ok(output), _) = shellout_result {
                let lines = output.split('\n');
                for line in lines {
                    if line.starts_with("Your system is consistent") {
                        chevrons::eerht(Color::Green);
                        println!("No broken reverse dependencies were found");
                        return true;
                    }
                }
            }
            chevrons::eerht(Color::Red);
            println!("Broken reverse dependencies were found. Initiating revdep-rebuild");
            false
        }
        Upgrade::Real => {
            let shellout_result = linux::system_command_interactive(
                "revdep-rebuild",
                "Rebuilding reverse dependencies",
            );
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
    let shellout_result =
        linux::system_command_non_interactive(
            "eix-test-obsolete", 
            "Checking obsolete configs"
            );
    linux::exit_on_failure(&shellout_result);
}

// This function cleans up old kernels
pub fn eclean_kernel() {
    let shellout_result =
        linux::system_command_interactive(
            "eclean-kernel -Aa", 
            "Cleaning old kernels"
            );
    linux::exit_on_failure(&shellout_result);
}

// This function removes old unused package tarballs
//
pub fn eclean_distfiles() {
    let shellout_result =
        linux::system_command_interactive(
            "eclean -d distfiles", 
            "Cleaning unused distfiles"
            );
    linux::exit_on_failure(&shellout_result);
}

pub fn eix_update() {
    let shellout_result = linux::system_command_quiet("eix-update");
    linux::exit_on_failure(&shellout_result);
}
pub fn handle_news() -> u32 {
    let mut count: u32 = 0;
    let shellout_result = linux::system_command_quiet("eselect news count new");
    linux::exit_on_failure(&shellout_result);
    if let (Ok(output), _) = shellout_result {
        count = output.trim().parse().unwrap_or(0);
    }
    count
}
