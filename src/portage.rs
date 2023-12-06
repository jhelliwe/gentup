use std::fs;
use filetime::FileTime;
use std::process::{Command};
use crate::prompt::*;
use crate::PromptType;
use crate::system_command::*;

// This function checks if the last portage sync was too recent (<=24 hours ago)
//
pub fn too_recent() -> bool {
    let portage_metadata = fs::metadata("/var/db/repos/gentoo/metadata/timestamp").unwrap();
    let filestamp = FileTime::from_last_modification_time(&portage_metadata).seconds() ;
    let nowutc = chrono::offset::Utc::now();
    let nowstamp = nowutc.timestamp();

    if nowstamp - filestamp < (24 * 60 * 60) {
        println!("Checking timestamp of last sync\t\t Skipping sync phase");
        return true;
    } else {
        return false;
    }
}

// This functions checks that a named package is installed. 
//
pub fn package_is_missing(package: &str) -> bool {
    let process = Command::new("eix")
    .arg(package)
    .output()
    .expect("EIX");
    let output = String::from_utf8_lossy(&process.stdout);
    if output.eq("No matches found\n") {
        println!("{} is not installed", package);
        return true;
    }
    println!("Package {} is installed", package) ;
    false
}

// This functions updates the package tree metadata for Gentoo Linux
//
pub fn do_eix_sync() {
    system_command("eix-sync");
    ask_user("Please verify the output of eix-sync above", PromptType::PressCR);
}

// This functions calls eix to check if the package manager "portage" is due an upgrade, since we
// want to make sure that the sys-apps/portage package is updated before all others!
//
pub fn portage_outdated() -> bool {
    print!("Checking portage version \t\t");
    let process = Command::new("eix")
    .arg("-u")
    .arg("sys-apps/portage")
    .output()
    .expect("EIX PORTAGE");
    let output = String::from_utf8_lossy(&process.stdout);
    if output.eq("No matches found\n") {
        println!(" sys-apps/portage is up to date");
        return false;
    }
    println!("Portage needs ugrade");
    true
}

// This functions performs an update of the sys-apps/portage package
//
pub fn upgrade_portage() {
        system_command("emerge --quiet -1av portage");
}

// This function performs an update of the world set - i.e a full system upgrade
//
pub fn upgrade_world() {
        system_command("emerge --quiet -auNDv --with-bdeps y --changed-use --complete-graph @world"); 
}

// This function does a depclean
//
pub fn depclean(run_type: crate::Upgrade) {
    match run_type {
        crate::Upgrade::Pretend => {
            println!("Performing dependency check... Please wait");
            system_command("emerge -p --depclean");
        },

        crate::Upgrade::Real => {
           system_command("emerge --depclean");
           ask_user("Please verify the output of emerge --depclean above", PromptType::PressCR);
        }
    }
}

// This functions calls revdep-rebuild which scans for broken reverse dependencies
pub fn revdep_rebuild(run_type: crate::Upgrade) {
    match run_type {
        crate::Upgrade::Pretend => { 
            println!("Performing reverse dependency check... Please wait");
            system_command("revdep-rebuild -pq");
        },
        crate::Upgrade::Real => { 
            system_command("revdep-rebuild");
            ask_user("Please verify the output of revdep-rebuild above", PromptType::PressCR);
        },
    }
}

// This functions calls the portage sanity checker
pub fn eix_test_obsolete() {
        println!("Performing portage hygiene tests");
        system_command("eix-test-obsolete");
}

// This functions cleans up old kernels
pub fn eclean_kernel() {
        system_command("eclean-kernel -Aa");
}

// This functions removes old unused package tarballs
//
pub fn eclean_distfiles() {
        system_command("eclean -d distfiles");
}
