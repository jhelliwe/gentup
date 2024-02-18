use crate::{
    chevrons, linux, portage,
    prompt::ask_user,
    tabulate,
    CmdVerbose::*,
    DepClean, PromptType::{self, PressCR}, RevDep, Upgrade,
};
use crossterm::{cursor, execute, style::Color};
use filetime::FileTime;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Seek, SeekFrom, Write},
    path::Path,
    process,
};
use terminal_spinners::{SpinnerBuilder, LINE};

// If the are no packages to update, return false. Or return true otherwise
// and also display a list of packages pending updates
pub fn portage_diff(fetch: bool) -> bool {
    let shellout_result = linux::system_command(
        "emerge -puDv @world",
        "Checking for updates",
        NonInteractive,
    );
    linux::exit_on_failure(&shellout_result);
    match shellout_result {
        (Ok(output), _) => {
            let mut pending_updates = Vec::new();
            for line in output.split('\n') {
                if line.starts_with("[ebuild") {
                    let mut words = line.split_whitespace();
                    let mut _word: Option<&str> = Some("");
                    for _counter in 1..=8 {
                        _word = words.next();
                        if _word.eq(&Some("]")) {
                            break
                        }
                    }
                    _word = words.next();
                    match _word {
                        Some(_word) => {
                            pending_updates.push(_word);
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
                    chevrons::eerht(Color::Blue);
                    println!("There are no pending updates");
                    return false;
                }
                1 => {
                    chevrons::eerht(Color::Yellow);
                    println!("There is 1 package pending an update");
                }
                _ => {
                    chevrons::eerht(Color::Yellow);
                    println!("There are {} packages pending updates", num_updates);
                }
            }
            tabulate::package_list(&pending_updates);
            if fetch {
                portage::fetch_sources(&pending_updates);
            }
            true
        }
        (Err(_), _) => {
            chevrons::three(Color::Red);
            eprintln!("Error calling emerge");
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
        chevrons::eerht(Color::Yellow);
        println!("Last sync was too recent: Skipping sync phase");
        true
    } else {
        false
    }
}

// This function checks that a named package is installed.
//
pub fn package_is_missing(package: &str) -> bool {
    let shellout_result = linux::system_command(&["equery l ", package].concat(), "", Quiet);
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
    let shellout_result = linux::system_command("eix-sync", "Syncing package tree", NonInteractive);
    linux::exit_on_failure(&shellout_result);
}

// This function calls eix to check if the named package is due an upgrade
//
pub fn package_outdated(package: &str) -> bool {
    let shellout_result = linux::system_command(&["eix -u ", package].concat(), "", Quiet);
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
    let shellout_result = linux::system_command(
        &["emerge --quiet -1av ", package].concat(),
        "Upgrading package",
        Interactive,
    );
    linux::exit_on_failure(&shellout_result);
}

// This function performs an update of the world set - i.e a full system upgrade
// It can optionally run in fetch mode, whereby it merely downloads the ebuilds instead of
// installing them
pub fn upgrade_world(run_type: Upgrade) {
    match run_type {
        Upgrade::Real => {
            let shellout_result = linux::system_command(
                "emerge --quiet -uNDv --with-bdeps y --changed-use --complete-graph @world",
                "Updating world set",
                Interactive,
            );
            linux::exit_on_failure(&shellout_result);
        }
        Upgrade::Fetch => {
            let shellout_result = linux::system_command(
                "emerge --quiet --fetchonly -uNDv --with-bdeps y --changed-use --complete-graph @world",
                "Fetching sources",
                NonInteractive,
            );
            linux::exit_on_failure(&shellout_result);
        }
        _ => {} // Future expansion
    }
}

// After package installs there are sometimes messages to the user advising them of actions they
// need to take. These are collected into elog files and displayed here
pub fn elogv() {
    let _shellout_result =
        linux::system_command("elogv", "Checking for new ebuild logs", Interactive);
}

// This function does a depclean
//
pub fn depclean(run_type: DepClean) -> (i32, i32) {
    let mut kernels = 0;
    match run_type {
        DepClean::Pretend | DepClean::KernelPretend => {
            let mut _depclean_command = String::new();
            if run_type == DepClean::Pretend {
                _depclean_command = "emerge -p --depclean --exclude sys-kernel/gentoo-kernel-bin --exclude sys-kernel/gentoo-sources".to_string();
            } else {
                _depclean_command = "emerge -p --depclean".to_string();
            }
            let shellout_result = linux::system_command(
                &_depclean_command,
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
                                if numdep == 0 {
                                    chevrons::eerht(Color::Blue);
                                } else {
                                    chevrons::eerht(Color::Yellow);
                                }
                                println!("Found {} dependencies to clean, {} of which is a kernel package", numdep, kernels);
                                return (numdep, kernels);
                            }
                            None => {
                                chevrons::eerht(Color::Green);
                                println!("There are no orphamed dependencies");
                                return (0, kernels);
                            }
                        }
                    }
                }
            }

            (0, 0)
        }

        DepClean::Real => {
            let shellout_result = linux::system_command(
                "emerge -a --depclean --exclude sys-kernel/gentoo-kernel-bin --exclude sys-kernel/gentoo-sources",
                "Removing orphaned dependencies",
                Interactive,
            );
            linux::exit_on_failure(&shellout_result);
            ask_user(
                "Please verify the output of emerge --depclean above",
                PromptType::PressCR,
            );
            (0, 0)
        }

        DepClean::Kernel => {
            let shellout_result = linux::system_command(
                "emerge --ask --depclean",
                "Removing orphaned dependencies",
                Interactive,
            );
            linux::exit_on_failure(&shellout_result);
            ask_user(
                "Please verify the output of emerge --depclean above",
                PromptType::PressCR,
            );
            (0, 0)
        }
        _ => (0, 0),
    }
}

// Reverse dependency check
pub fn revdep_rebuild(run_type: RevDep) -> bool {
    match run_type {
        RevDep::Pretend => {
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
                        chevrons::eerht(Color::Blue);
                        println!("No broken reverse dependencies were found");
                        return true;
                    }
                }
            }
            chevrons::eerht(Color::Yellow);
            println!("Broken reverse dependencies were found. Initiating revdep-rebuild");
            false
        }
        RevDep::Real => {
            let shellout_result = linux::system_command(
                "revdep-rebuild",
                "Rebuilding reverse dependencies",
                Interactive,
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

// This function calls the portage config sanity checker
pub fn eix_test_obsolete() {
    let shellout_result = linux::system_command(
        "eix-test-obsolete",
        "Checking obsolete configs",
        Interactive,
    );
    linux::exit_on_failure(&shellout_result);
}

// This function cleans up old kernels
pub fn eclean_kernel() {
    let shellout_result =
        linux::system_command("eclean-kernel -Aa", "Cleaning old kernels", Interactive);
    linux::exit_on_failure(&shellout_result);
}

// This function removes old unused package tarballs
//
pub fn eclean_distfiles() {
    let shellout_result = linux::system_command(
        "eclean -d distfiles",
        "Cleaning unused distfiles",
        Interactive,
    );
    linux::exit_on_failure(&shellout_result);
}

// eix_update resynchronises the eix database with the state of the currently installed packages
pub fn eix_update() {
    let shellout_result = linux::system_command(
        "eix-update",
        "Initialising package database",
        NonInteractive,
    );
    linux::exit_on_failure(&shellout_result);
}

// handle_news checks to see if there is unread news and lists it if required
pub fn handle_news() -> u32 {
    let mut count: u32 = 0;
    let shellout_result = linux::system_command("eselect news count new", "", Quiet);
    linux::exit_on_failure(&shellout_result);
    if let (Ok(output), _) = shellout_result {
        count = output.trim().parse().unwrap_or(0);
        if count == 0 {
            chevrons::eerht(Color::Blue);
            println!("No news is good news")
        } else {
            chevrons::eerht(Color::Yellow);
            println!("You have {} news item(s) to read", count);
            let _ignore = linux::system_command("eselect news list", "News listing", Interactive);
            let _ignore = linux::system_command("eselect news read", "News listing", Interactive);
        }
    }
    count
}

// dispatch_conf handles pending changes to package configuration files
pub fn dispatch_conf() {
    let shellout_result =
        linux::system_command("dispatch-conf", "Merge config file changes", Interactive);
    linux::exit_on_failure(&shellout_result);
}

// Checks and corrects the ELOG configuration in make.conf
pub fn elog_make_conf() {
    let makeconf = fs::read_to_string("/etc/portage/make.conf");
    if let Ok(contents) = makeconf {
        for eachline in contents.lines() {
            if eachline.contains("PORTAGE_ELOG_SYSTEM") {
                return;
            }
        }
        chevrons::three(Color::Yellow);
        println!("Configuring elogv");
        let mut file = OpenOptions::new()
            .write(true)
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
            chevrons::eerht(Color::Yellow);
            println!("This updater requires the {} package.", &package[0]);
            let shellout_result = linux::system_command(
                &["emerge --quiet -v ", &package[0]].concat(),
                &["Installing ", &package[0]].concat(),
                NonInteractive,
            );
            linux::exit_on_failure(&shellout_result);
            if !&package[2].eq("") {
                let shellout_result = linux::system_command(
                    package[2],
                    "Post installation configuration",
                    NonInteractive,
                );
                linux::exit_on_failure(&shellout_result);
            }
        }
    }

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
        chevrons::eerht(Color::Red);
        println!("No /etc/default/gentup configuration file detected");
        chevrons::eerht(Color::Red);
        println!("Creating /etc/default/gentup with a default package list");
        chevrons::eerht(Color::Red);
        println!("These packages will be installed by this updater");
        chevrons::eerht(Color::Red);
        println!("Please customise this list to your preferences, and then re-run this program");
        process::exit(1);
    }

    // Read /etc/default/gentup into a Vector of strings
    let packages_to_check_string =
        fs::read_to_string("/etc/default/gentup").expect("Error in reading the file");
    let mut counter = 0;
    let packages_to_check: Vec<&str> = packages_to_check_string.lines().collect();
    for check in &packages_to_check {
        counter += 1;
        chevrons::eerht(Color::Green);
        println!(
            "Checking prerequsite package : {} of {} - {}                    ",
            counter,
            packages_to_check.len(),
            check
        );
        let _ignore = execute!(io::stdout(), cursor::MoveUp(1));
        if portage::package_is_missing(check) {
            println!("                                                      ");
            chevrons::eerht(Color::Yellow);
            println!(
                "This program requires {} to be installed. Installing...",
                check
            );
            let cmdline = [
                "emerge --quiet --autounmask y --autounmask-write y -av ",
                check,
            ]
            .concat();
            let shellout_result =
                linux::system_command(&cmdline, "Installing missing package", Interactive);
            linux::exit_on_failure(&shellout_result);
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

        let shellout_result = linux::system_command(
            &["emerge --fetchonly --nodeps =", ebuild_to_fetch].concat(),
            "",
            Quiet,
        );
        linux::exit_on_failure(&shellout_result);
        handle.stop_and_clear();
    }
    ask_user("Press CR", PressCR);
}
