use crate::linux;
use crate::prompt::*;
use crate::PromptType;
use crate::Upgrade;
use filetime::FileTime;
use std::fs;
use std::process;
use std::io::Write;

pub fn eix_diff() -> bool {
    let shellout_result = linux::system_command_quiet("eix-diff");
    linux::exit_on_failure(&shellout_result);
    match shellout_result {
        (Ok(output), _) => {
            let mut pending_updates = Vec::new();
            for line in output.split("\n") {
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
                    println!("<<< There are no pending updates");
                    return false;
                }
                1 => {
                    println!("<<< There is 1 package pending an update");
                }
                _ => {
                    println!("\n<<< There are {} packages pending updates", num_updates);
                }
            }
            for item in pending_updates {
                print!("{}   ", item);
            }
            println!();
            return true;
        }
        (Err(_), _) => {
            eprintln!("<<< Error calling eix-diff");
            return false;
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
        println!(">>> Last sync was too recent: Skipping sync phase");
        return true;
    } else {
        return false;
    }
}

// This functions checks that a named package is installed.
//
pub fn package_is_missing(package: &str) -> bool {
    print!(".");
    std::io::stdout().flush().unwrap();
    let shellout_result = linux::system_command_quiet(&["equery l ", package].concat());
    match shellout_result {
        (Ok(_), return_code) => {
            if return_code != 0 {
                println!("\n<<< {} is not installed", package);
                return true;
            }
            false
        }
        (Err(returned_error), _) => {
            eprintln!("\n<<< Problem running command: {}", returned_error);
            process::exit(1);
        }
    }
}

// This functions updates the package tree metadata for Gentoo Linux
//
pub fn do_eix_sync() {
    println!(">>> Downloading latest package tree - please wait");
    let shellout_result = linux::system_command("eix-sync -q");
    linux::exit_on_failure(&shellout_result);
}

// This functions calls eix to check if the named package is due an upgrade
//
pub fn package_outdated(package: &str) -> bool {
    let shellout_result = linux::system_command_quiet(&["eix -u ", package].concat());
    match shellout_result {
        (Ok(_), return_status) => {
            if return_status != 0 {
                return false;
            }
            println!("<<< {} needs upgrade", package);
            return true;
        }
        (Err(_), returned_error) => {
            eprintln!("<<< Command returned {}", returned_error);
            process::exit(1);
        }
    }
}

// This functions performs an update of the named package
//
pub fn upgrade_package(package: &str) {
    let shellout_result = linux::system_command(&["emerge --quiet -1av ", package].concat());
    linux::exit_on_failure(&shellout_result);
}

// This function performs an update of the world set - i.e a full system upgrade
//
pub fn upgrade_world() {
    let shellout_result = linux::system_command(
        "emerge --quiet -auNDv --with-bdeps y --changed-use --complete-graph @world",
    );
    linux::exit_on_failure(&shellout_result);
}

// This function does a depclean
//
pub fn depclean(run_type: Upgrade) -> i32 {
    match run_type {
        Upgrade::Pretend => {
            println!(">>> Performing dependency check... Please wait");
            let shellout_result = linux::system_command_quiet("emerge -p --depclean");
            linux::exit_on_failure(&shellout_result);
            match shellout_result {
                (Ok(output), _) => {
                    let lines = output.split("\n");
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
                (Err(_), _) => {}
            }
            return 0;
        }

        Upgrade::Real => {
            let shellout_result = linux::system_command("emerge --depclean");
            linux::exit_on_failure(&shellout_result);
            ask_user(
                "Please verify the output of emerge --depclean above",
                PromptType::PressCR,
            );
            return 0;
        }
    }
}

pub fn revdep_rebuild(run_type: Upgrade) -> bool {
    match run_type {
        Upgrade::Pretend => {
            println!(">>> Performing reverse dependency check... Please wait");
            let shellout_result = linux::system_command_quiet("revdep-rebuild -ip");
            linux::exit_on_failure(&shellout_result);
            match shellout_result {
                (Ok(output),_) => {
                    let lines = output.split("\n");
                    for line in lines {
                        println!("{line}");
                        if line.starts_with("Your systen is consistent") {
                            return true;
                        }
                    }
                }
                (Err(_),_) => {
                }
            }
            return false;
        }
        Upgrade::Real => {
            let shellout_result = linux::system_command("revdep-rebuild");
            linux::exit_on_failure(&shellout_result);
            ask_user(
                "Please verify the output of revdep-rebuild above",
                PromptType::PressCR,
            );
            return true;
        }
    }
}

// This functions calls the portage sanity checker
pub fn eix_test_obsolete() {
    println!(">>> Performing portage hygiene tests");
    let shellout_result = linux::system_command("eix-test-obsolete");
    linux::exit_on_failure(&shellout_result);
}

// This functions cleans up old kernels
pub fn eclean_kernel() {
    let shellout_result = linux::system_command("eclean-kernel -Aa");
    linux::exit_on_failure(&shellout_result);
}

// This functions removes old unused package tarballs
//
pub fn eclean_distfiles() {
    let shellout_result = linux::system_command("eclean -d distfiles");
    linux::exit_on_failure(&shellout_result);
}
