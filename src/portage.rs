// This source file contains logic which interacts with the Portage package manager

use crate::{
    linux, portage,
    prompt::{self, ask_user},
    CmdVerbose::*,
    Orphans,
    PromptType::{self, PressCR},
    ShellOutResult,
    Upgrade::*,
};
use crossterm::{cursor, execute, style::Color};
use filetime::FileTime;
use regex::Regex;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Seek, SeekFrom, Write},
    path::Path,
    process,
};
use terminal_spinners::{SpinnerBuilder, LINE};

// Here are defined the behaviours of the package Upgrade methods
// Pretend lets the caller see what would be upgraded without actually performing an Upgrade
// Real and its derivatives do perform the actual package updates
#[derive(PartialEq)]
pub enum Upgrade {
    Pretend,
    Real,
    RealExcludeKernels,
    RealIncludeKernels,
}

impl Upgrade {
    // Implementation steps for performing a full system update
    pub fn all_packages(run_type: Self) -> ShellOutResult {
        match run_type {
            Real => linux::system_command(
                "emerge --quiet -uNDv --with-bdeps y --changed-use --complete-graph @world",
                "Updating world set",
                Interactive,
            ),
            Pretend => linux::system_command(
                "emerge -puDv @world",
                "Checking for updates",
                NonInteractive,
            ),
            _ => (Ok(String::new()), 0),
        }
    }

    // Implementation steps to clean orphaned packages
    pub fn clean(run_type: Self) -> Orphans {
        let mut kernels = 0;
        match run_type {
            Pretend => {
                let shellout_result = linux::system_command(
                    "emerge -p --depclean",
                    "Checking for orphaned dependencies",
                    NonInteractive,
                );
                linux::exit_on_failure(&shellout_result);
                if let (Ok(output), _) = shellout_result {
                    let lines = output.split('\n');
                    for line in lines {
                        if line.contains("kernel") {
                            kernels += 1;
                        }
                        if line.starts_with("Number to remove") {
                            let mut words = line.split_whitespace();
                            let mut word: Option<&str> = Some("");
                            for _counter in 0..=3 {
                                word = words.next();
                            }
                            match word {
                                Some(word) => {
                                    let numdep = word.parse().unwrap();
                                    let mut _depcolor = Color::Green;
                                    if numdep == 0 {
                                        _depcolor = Color::Blue;
                                    } else {
                                        _depcolor = Color::Yellow;
                                    }
                                    println!(
                                        "{} Found {} dependencies to clean",
                                        prompt::revchevrons(_depcolor),
                                        numdep
                                    );
                                    return (numdep, kernels);
                                }
                                None => {
                                    println!(
                                        "{} There are no orphamed dependencies",
                                        prompt::revchevrons(Color::Green)
                                    );
                                    return (0, kernels);
                                }
                            }
                        }
                    }
                }

                (0, 0)
            }

            RealExcludeKernels => {
                linux::exit_on_failure(&linux::system_command(
                "emerge -a --depclean --exclude sys-kernel/gentoo-kernel-bin --exclude sys-kernel/gentoo-sources",
                "Removing orphaned dependencies",
                Interactive,
            ));
                ask_user(
                    "Please verify the output of emerge --depclean above",
                    PromptType::PressCR,
                );
                (0, 0)
            }

            RealIncludeKernels => {
                linux::exit_on_failure(&linux::system_command(
                    "emerge --ask --depclean",
                    "Removing orphaned dependencies",
                    Interactive,
                ));
                ask_user(
                    "Please verify the output of emerge --depclean above",
                    PromptType::PressCR,
                );
                (0, 0)
            }
            _ => (0, 0),
        }
    }

    // Check for broken reverse dependences and rebuild
    pub fn rebuild(run_type: Self) -> bool {
        match run_type {
            Pretend => {
                let shellout_result = linux::system_command(
                    "revdep-rebuild -ip",
                    "Checking reverse dependencies",
                    NonInteractive,
                );
                linux::exit_on_failure(&shellout_result);
                if let (Ok(output), _) = shellout_result {
                    let lines = output.split('\n');
                    for line in lines {
                        if line.starts_with("Your system is consistent") {
                            println!(
                                "{} No broken reverse dependencies were found",
                                prompt::revchevrons(Color::Blue)
                            );
                            return true;
                        }
                    }
                }
                println!(
                    "{} Broken reverse dependencies were found. Initiating revdep-rebuild",
                    prompt::revchevrons(Color::Yellow)
                );
                false
            }
            Real => {
                linux::exit_on_failure(&linux::system_command(
                    "revdep-rebuild",
                    "Rebuilding reverse dependencies",
                    Interactive,
                ));
                ask_user(
                    "Please verify the output of revdep-rebuild above",
                    PromptType::PressCR,
                );
                true
            }
            _ => false,
        }
    }
}

// List and fetch pending updates. Returns "true" if there are any pending updates
// Returns false if there are no pending updates
pub fn get_pending_updates(fetch: bool) -> bool {
    match Upgrade::all_packages(Pretend) {
        (Ok(output), _) => {
            let mut pending_updates = Vec::new();
            for line in output.split('\n') {
                if line.starts_with("[ebuild") {
                    let mut words = line.split(']');
                    let _word = words.next();
                    let _word = words.next();
                    match _word {
                        Some(_word) => {
                            let word = _word.split_whitespace().next().unwrap_or("");
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
                    println!(
                        "{} There are no pending updates",
                        prompt::revchevrons(Color::Blue)
                    );
                    return false;
                }
                1 => {
                    println!(
                        "{} There is 1 package pending an update",
                        prompt::revchevrons(Color::Yellow)
                    );
                }
                _ => {
                    println!(
                        "{} There are {} packages pending updates",
                        prompt::revchevrons(Color::Yellow),
                        num_updates
                    );
                }
            }
            portage::package_list(&pending_updates);
            if fetch {
                portage::fetch_sources(&pending_updates);
            }
            true
        }
        (Err(_), _) => {
            eprintln!("{} Error calling emerge", prompt::revchevrons(Color::Red));
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
        println!(
            "{} Last sync was too recent: Skipping sync phase",
            prompt::revchevrons(Color::Yellow)
        );
        true
    } else {
        false
    }
}

// This function checks that a named package is installed.
//
pub fn package_is_missing(package: &str) -> bool {
    match linux::system_command(&["equery l ", package].concat(), "", Quiet) {
        (Ok(_), return_code) => {
            if return_code != 0 {
                println!();
                println!(
                    "{} {} is not installed",
                    prompt::revchevrons(Color::Yellow),
                    package
                );
                return true;
            }
            false
        }
        (Err(returned_error), _) => {
            eprintln!();
            eprintln!(
                "{} Problem running command: {}",
                prompt::revchevrons(Color::Red),
                returned_error
            );
            process::exit(1);
        }
    }
}

// This function updates the package tree metadata for Gentoo Linux
//
pub fn sync_package_tree() {
    linux::exit_on_failure(&linux::system_command(
        "eix-sync",
        "Syncing package tree",
        NonInteractive,
    ));
}

// This function calls eix to check if the named package is due an upgrade
//
pub fn package_outdated(package: &str) -> bool {
    match linux::system_command(&["eix -u ", package].concat(), "", Quiet) {
        (Ok(_), return_status) => {
            if return_status != 0 {
                return false;
            }
            println!(
                "{} {} needs to be upgraded",
                prompt::revchevrons(Color::Yellow),
                package
            );
            true
        }
        (Err(_), returned_error) => {
            eprintln!(
                "{} Command returned {}",
                prompt::revchevrons(Color::Red),
                returned_error
            );
            process::exit(1);
        }
    }
}

// This function performs an update of the named package
//
pub fn upgrade_package(package: &str) {
    linux::exit_on_failure(&linux::system_command(
        &["emerge --quiet -1v ", package].concat(),
        "Upgrading package",
        Interactive,
    ));
}

// After package installs there are sometimes messages to the user advising them of actions they
// need to take. These are collected into elog files and displayed here
pub fn elog_viewer() {
    let _shellout_result =
        linux::system_command("elogv", "Checking for new ebuild logs", Interactive);
}

// This function calls the portage config sanity checker
pub fn eix_test_obsolete() {
    linux::exit_on_failure(&linux::system_command(
        "eix-test-obsolete",
        "Checking obsolete configs",
        Interactive,
    ));
}

// This function cleans up old kernels
pub fn eclean_kernel() {
    linux::exit_on_failure(&linux::system_command(
        "eclean-kernel -Aa",
        "Cleaning old kernels",
        Interactive,
    ));
}

// This function removes old unused package tarballs
//
pub fn eclean_distfiles() {
    linux::exit_on_failure(&linux::system_command(
        "eclean -d distfiles",
        "Cleaning unused distfiles",
        Interactive,
    ));
}

// eix_update resynchronises the eix database with the state of the currently installed packages
pub fn eix_update() {
    linux::exit_on_failure(&linux::system_command(
        "eix-update",
        "Initialising package database",
        NonInteractive,
    ));
}

// handle_news checks to see if there is unread news and lists it if required
pub fn read_news() -> u32 {
    let mut count: u32 = 0;
    let shellout_result = linux::system_command("eselect news count new", "", Quiet);
    linux::exit_on_failure(&shellout_result);
    if let (Ok(output), _) = shellout_result {
        count = output.trim().parse().unwrap_or(0);
        if count == 0 {
            println!("{} No news is good news", prompt::revchevrons(Color::Blue));
        } else {
            println!(
                "{} You have {} news item(s) to read",
                prompt::revchevrons(Color::Yellow),
                count,
            );
            let _ignore = linux::system_command("eselect news list", "News listing", Interactive);
            let _ignore = linux::system_command("eselect news read", "News listing", Interactive);
        }
    }
    count
}

// dispatch_conf handles pending changes to package configuration files
pub fn update_config_files() {
    linux::exit_on_failure(&linux::system_command(
        "dispatch-conf",
        "Merge config file changes",
        Interactive,
    ));
}

// Checks and corrects the ELOG configuration in make.conf
pub fn configure_elogv() {
    let makeconf = fs::read_to_string("/etc/portage/make.conf");
    if let Ok(contents) = makeconf {
        for eachline in contents.lines() {
            if eachline.contains("PORTAGE_ELOG_SYSTEM") {
                return;
            }
        }
        println!("{} Configuring elogv", prompt::chevrons(Color::Yellow));
        let mut file = OpenOptions::new()
            .append(true)
            .open("/etc/portage/make.conf")
            .unwrap();
        file.seek(SeekFrom::End(0)).unwrap();
        file.write_all(
            b"# Logging\nPORTAGE_ELOG_CLASSES=\"warn error log\"\nPORTAGE_ELOG_SYSTEM=\"save\"\n",
        )
        .unwrap();
    }
}

pub fn check_and_install_deps() {
    // Check the hard dependencies of this program
    let packages_to_check = [
        ["app-portage/eix", "/usr/bin/eix", "eix-update"],
        ["app-portage/gentoolkit", "/usr/bin/equery", ""],
        ["app-portage/elogv", "/usr/bin/elogv", ""],
        ["app-admin/eclean-kernel", "/usr/bin/eclean-kernel", ""],
    ];

    for package in packages_to_check {
        if !Path::new(&package[1]).exists() {
            println!(
                "{} This updater requires the {} package.",
                prompt::revchevrons(Color::Yellow),
                &package[0]
            );
            linux::exit_on_failure(&linux::system_command(
                &["emerge --quiet -v ", &package[0]].concat(),
                &["Installing ", &package[0]].concat(),
                NonInteractive,
            ));
            if !&package[2].eq("") {
                linux::exit_on_failure(&linux::system_command(
                    package[2],
                    "Post installation configuration",
                    NonInteractive,
                ));
            }
        }
    }
}

pub fn check_and_install_optional_packages() {
    // This following list of packages is hardcoded. While this is good for me, other users may be annoyed at
    // this personal choice. So we get this list read in from a text file. Then the user can modify
    // it to their requirements. And if the file does not exist, pre-populate it anyway

    let packages_to_check = [
        "app-portage/cpuid2cpuflags",
        "app-portage/pfl",
        "app-portage/ufed",
        "app-admin/sysstat",
        "app-editors/vim",
        "net-dns/bind-tools",
        "app-misc/tmux",
        "net-misc/netkit-telnetd",
        "sys-apps/mlocate",
        "sys-apps/inxi",
        "sys-apps/pciutils",
        "sys-apps/usbutils",
        "sys-process/nmon",
        "dev-lang/rust-bin",
        "dev-vcs/git",
    ];

    // If /etc/default/gentup does not exist, create it with the above contents
    if !Path::new("/etc/default/gentup").exists() {
        let path = Path::new("/etc/default/gentup");
        let display = path.display();
        let mut file = match File::create(path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };
        for check in packages_to_check {
            match writeln!(file, "{check}") {
                Err(why) => panic!("couldn't write to {}: {}", display, why),
                Ok(file) => file,
            }
        }
        println!(
            "{} No /etc/default/gentup configuration file detected",
            prompt::revchevrons(Color::Red),
        );
        println!(
            "{} Creating /etc/default/gentup with a default package list",
            prompt::revchevrons(Color::Red),
        );
        println!(
            "{} These packages will be installed by this updater",
            prompt::revchevrons(Color::Red),
        );
        println!(
            "{} Please customise this list to your preferences, and then re-run this program",
            prompt::revchevrons(Color::Red),
        );
        process::exit(1);
    }

    // Read /etc/default/gentup into a Vector of strings
    let packages_to_check_string =
        fs::read_to_string("/etc/default/gentup").expect("Error in reading the file");
    let mut counter = 0;
    let packages_to_check: Vec<&str> = packages_to_check_string.lines().collect();
    for check in &packages_to_check {
        counter += 1;
        println!(
            "{} Checking prerequsite package : {} of {} - {}                    ",
            prompt::revchevrons(Color::Green),
            counter,
            packages_to_check.len(),
            check
        );
        let _ignore = execute!(io::stdout(), cursor::MoveUp(1));
        if portage::package_is_missing(check) {
            println!("                                                      ");
            println!(
                "{} This program requires {} to be installed. Installing...",
                prompt::revchevrons(Color::Yellow),
                check
            );
            let cmdline = [
                "emerge --quiet --autounmask y --autounmask-write y -av ",
                check,
            ]
            .concat();
            linux::exit_on_failure(&linux::system_command(
                &cmdline,
                "Installing missing package",
                Interactive,
            ));
        }
    }
    println!("                                                                   ");
    let _ignore = execute!(io::stdout(), cursor::MoveUp(1));
}

pub fn fetch_sources(package_vec: &Vec<&str>) {
    let mut count = 0;
    let total = package_vec.len();
    for ebuild_to_fetch in package_vec {
        count += 1;
        let text = [
            " Downloading ",
            &count.to_string(),
            " of ",
            &total.to_string(),
            ": ",
            ebuild_to_fetch,
        ]
        .concat();
        let handle = SpinnerBuilder::new().spinner(&LINE).text(text).start();
        linux::exit_on_failure(&linux::system_command(
            &["emerge --fetchonly --nodeps =", ebuild_to_fetch].concat(),
            "",
            Quiet,
        ));
        handle.done();
    }
    ask_user("Downloads complete", PressCR);
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
    let (width, _height) = linux::termsize();
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
