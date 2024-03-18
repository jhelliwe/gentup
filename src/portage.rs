// This source file contains logic which interacts with the Portage package manager

use crate::{
    linux, linux::CouldFail, linux::OsCall, linux::ShellOutResult, portage, prompt, Prompt,
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

// Describe actions to be taken with the package manager
#[derive(PartialEq)]
pub enum Gentoo {
    DryRun,
    FullRun,
    PreserveKernel,
    AllPackages,
}

// Describe orphaned packages
pub type Orphans = (i32, String);

// Describe methods used with package manager tools
impl Gentoo {
    // Perform an update of the @world set (full system update)
    pub fn update_all_packages(self) -> ShellOutResult {
        match self {
            Gentoo::FullRun => OsCall::Interactive.execute(
                "emerge --quiet -uNDv --with-bdeps y --changed-use --complete-graph @world",
                "Updating world set",
            ),
            Gentoo::DryRun => {
                OsCall::Spinner.execute("emerge -puDv @world", "Checking for updates")
            }
            _ => Ok((String::new(), 0)),
        }
    }

    // Check and clean orphaned packages, for example if php was installed and libgd was enabled,
    // php would have pulled in libgd as a dependency. If the user removes php, libgd is not
    // automatically removed. The depclean method here will detect libgd as an orphaned package and
    // will remove it.
    pub fn depclean(self) -> Orphans {
        let mut kernels = String::new();
        match self {
            Gentoo::DryRun => {
                if let Ok((output, _)) = OsCall::Spinner
                    .execute("emerge -p --depclean", "Checking for orphaned dependencies")
                    .exit_if_failed()
                {
                    let lines = output.split('\n');
                    for line in lines {
                        if line.contains("gentoo-kernel") || line.contains("gentoo-sources") {
                            kernels = linux::stripchar(line.to_string());
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

                (0, String::new())
            }
            Gentoo::PreserveKernel => {
                let _ = OsCall::Interactive.execute(
                "emerge -a --depclean --exclude sys-kernel/gentoo-kernel-bin --exclude sys-kernel/gentoo-sources",
                "Removing orphaned dependencies",
            ).exit_if_failed();
                Prompt::PressReturn.askuser("Please verify the output of emerge --depclean above");
                (0, String::new())
            }
            Gentoo::AllPackages => {
                let _ = OsCall::Interactive
                    .execute(
                        "emerge --ask --depclean",
                        "Removing all orphaned dependencies",
                    )
                    .exit_if_failed();
                Prompt::PressReturn.askuser("Please verify the output of emerge --depclean above");
                (0, String::new())
            }
            _ => (0, String::new()),
        }
    }

    // Check for broken reverse dependences and rebuild. For example if golang is updated, packages
    // that use golang (like k8s) would have to be reinstalled, because golang updates cause breakage.
    // revdep-rebuild is a relic, coming from a time when Portage didn't do it's own rebuild
    // checking - BUT sometimes Portage misses things. It's always a good idea to go through each
    // installed package and check that the dynamic libraries for each binary resolve and can be
    // linked at run-time
    pub fn revdep_rebuild(self) -> bool {
        match self {
            Gentoo::DryRun => {
                if let Ok((output, _)) = OsCall::Spinner
                    .execute("revdep-rebuild -ip", "Checking reverse dependencies")
                    .exit_if_failed()
                {
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
            Gentoo::FullRun => {
                let _ = OsCall::Interactive
                    .execute("revdep-rebuild", "Rebuilding reverse dependencies")
                    .exit_if_failed();
                Prompt::PressReturn.askuser("Please verify the output of revdep-rebuild above");
                true
            }
            _ => false,
        }
    }
}

// List and fetch pending updates. Returns "true" if there are any pending updates
// Returns false if there are no pending updates.
pub fn get_pending_updates(background_fetch: bool) -> bool {
    match Gentoo::DryRun.update_all_packages() {
        Ok((output, _)) => {
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
            if !background_fetch {
                portage::fetch_sources(&pending_updates);
            }
            true
        }
        Err(_) => {
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
    match OsCall::Quiet.execute(&["equery l ", package].concat(), "") {
        Ok((_, return_code)) => {
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
        Err(returned_error) => {
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
    let _ = OsCall::Spinner
        .execute("eix-sync", "Syncing package tree")
        .exit_if_failed();
}

// This function calls eix to check if the named package is due an upgrade
//
pub fn package_outdated(package: &str) -> bool {
    match OsCall::Quiet.execute(&["eix -u ", package].concat(), "") {
        Ok((_, return_status)) => {
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
        Err(returned_error) => {
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
    let _ = OsCall::Interactive
        .execute(
            &["emerge --quiet -1v ", package].concat(),
            "Upgrading package",
        )
        .exit_if_failed();
}

// After package installs there are sometimes messages to the user advising them of actions they
// need to take. These are collected into elog files and displayed here
pub fn elog_viewer() {
    let _ = OsCall::Interactive.execute("elogv", "Checking for new ebuild logs");
}

// This function calls the portage config sanity checker
pub fn eix_test_obsolete() {
    let _ = OsCall::Interactive
        .execute("eix-test-obsolete", "Checking obsolete configs")
        .exit_if_failed();
}

// This function cleans up old kernels
pub fn eclean_kernel() {
    let _ = OsCall::Interactive
        .execute("eclean-kernel -Aa", "Cleaning old kernels")
        .exit_if_failed();
}

// This function removes old unused package tarballs
//
pub fn eclean_distfiles() {
    let _ = OsCall::Interactive
        .execute("eclean -d distfiles", "Cleaning unused distfiles")
        .exit_if_failed();
}

// eix_update resynchronises the eix database with the state of the currently installed packages
pub fn eix_update() {
    let _ = OsCall::Spinner
        .execute("eix-update", "Initialising package database")
        .exit_if_failed();
}

// handle_news checks to see if there is unread news and lists it if required
pub fn read_news() -> u32 {
    let mut count: u32 = 0;
    if let Ok((output, _)) = OsCall::Quiet
        .execute("eselect news count new", "")
        .exit_if_failed()
    {
        count = output.trim().parse().unwrap_or(0);
        if count == 0 {
            println!("{} No news is good news", prompt::revchevrons(Color::Blue));
        } else {
            println!(
                "{} You have {} news item(s) to read",
                prompt::revchevrons(Color::Yellow),
                count,
            );
            let _ = OsCall::Interactive.execute("eselect news list", "News listing");
            let _ = OsCall::Interactive.execute("eselect news read", "News listing");
        }
    }
    count
}

// dispatch_conf handles pending changes to package configuration files
pub fn update_config_files() {
    let _ = OsCall::Interactive
        .execute("dispatch-conf", "Merge config file changes")
        .exit_if_failed();
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
            let _ = OsCall::Spinner
                .execute(
                    &["emerge --quiet -v ", &package[0]].concat(),
                    &["Installing ", &package[0]].concat(),
                )
                .exit_if_failed();
            if !&package[2].eq("") {
                let _ = OsCall::Spinner
                    .execute(package[2], "Post installation configuration")
                    .exit_if_failed();
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
        let _ = execute!(io::stdout(), cursor::MoveUp(1));
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
            let _ = OsCall::Interactive
                .execute(&cmdline, "Installing missing package")
                .exit_if_failed();
        }
    }
    println!("                                                                   ");
    let _ = execute!(io::stdout(), cursor::MoveUp(1));
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
        let _ = OsCall::Quiet
            .execute(
                &["emerge --fetchonly --nodeps =", ebuild_to_fetch].concat(),
                "",
            )
            .exit_if_failed();
        handle.done();
    }
}

// Shortens a package name for more aesthetic display to user
// e.g sys-cluster/kube-scheduler-1.29.1::gentoo to sys-cluster/kube-scheduler
pub fn shortname(packagename: &str) -> String {
    let mut position = packagename.len();
    let mut _previous = ' ';
    for (i, c) in packagename.chars().enumerate() {
        if c.is_numeric() && _previous == '-' {
            position = i;
            break;
        }
        _previous = c;
    }
    packagename[0..position - 1].to_string()
}

// Calculates the longest length of shortened package names in a vector of absolute package names
pub fn longest(vec_of_strings: &Vec<&str>) -> u16 {
    let mut longest_length = 0;
    let mut _thislen = 0;
    for string_to_consider in vec_of_strings {
        let shortened_string = shortname(string_to_consider);
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
        let shortitem = shortname(item);
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
