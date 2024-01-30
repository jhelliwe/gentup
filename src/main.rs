// Gentoo Updater
// John Helliwell

/* This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
*/

const VERSION: &str = "0.14a";

pub mod chevrons;
pub mod linux;
pub mod portage;
pub mod prompt;
pub mod tabulate;
use ansi_term::Colour;
use std::env;
use std::path::Path;
use std::process;

pub enum Upgrade {
    Real,
    Pretend,
    Fetch,
}

#[derive(PartialEq)]
pub enum PromptType {
    ClearScreen,
    Review,
    PressCR,
}

fn main() {
    match env::var("USER") {
        Ok(val) => {
            if val != "root" {
                eprintln!("You need to be root to run this");
                process::exit(1);
            }
        }
        Err(_) => {
            eprintln!("You need to be root to run this");
            process::exit(1);
        }
    }
    let args = env::args();
    let mut force: bool = false;
    let mut cleanup: bool = false;
    let mut first = true;
    for arg in args {
        if first {
            first = false;
            continue;
        }
        match &arg[..] {
            "-h" | "--help" => {
                println!("Usage:\n\n \
                    gentup [options]\n \
                    Options:\n\n\
                    -c, --cleanup  Only perform cleanup, useful if you interupped the previous run\n\
                    -f, --force    Force eix-sync, bypassing the timestamp check\n\
                    -h, --help     Display this help text, then exit\n\
                    -V, --version  Display the program version\
                ");
                process::exit(0);
            }
            "-V" | "--version" => {
                println!("gentup version {}", VERSION);
                process::exit(0);
            }
            "-f" | "--force" => {
                force = true;
            }
            "-c" | "--cleanup" => {
                cleanup = true;
            }
            _ => {
                eprintln!("Error: usage - gentup [--help|--force|--cleanup|--version]");
                process::exit(1);
            }
        }
    }

    let _ = clearscreen::clear();
    println!("\nWelcome to the Gentoo Updater v{}\n", VERSION);

    // Are we running on Gentoo?
    let _distro =
        linux::check_distro("Gentoo".to_string()).expect("This updater only works on Gentoo Linux");

    chevrons::three(Colour::Green);
    print!("Checking environment: ");
    // We won't get much further if eix is not installed. We must check this
    if !Path::new("/usr/bin/eix").exists() {
        let mut shellout_result = linux::system_command("emerge --quiet -v app-portage/eix");
        linux::exit_on_failure(&shellout_result);
        shellout_result = linux::system_command("eix-update");
        linux::exit_on_failure(&shellout_result);
    }

    // We won't get much further if equery is not installed. We must check this too
    if !Path::new("/usr/bin/equery").exists() {
        let shellout_result = linux::system_command("emerge --quiet -v app-portage/gentoolkit");
        linux::exit_on_failure(&shellout_result);
    }

    // Check some required (by me) packages are installed. Useful for a just-installed Gentoo
    let packages_to_check = [
        "app-portage/cpuid2cpuflags",
        "app-portage/elogv",
        "app-portage/pfl",
        "app-portage/ufed",
        "app-admin/eclean-kernel",
        "app-admin/sysstat",
        "net-dns/bind-tools",
        "app-misc/tmux",
        "net-misc/netkit-telnetd",
        "sys-apps/mlocate",
        "sys-apps/util-linux",
        "sys-process/nmon",
    ];
    for check in packages_to_check {
        if portage::package_is_missing(&check) {
            println!(
                "<<< This program requires {} to be installed. Installing...",
                check
            );
            let cmdline = [
                "emerge --quiet --autounmask y --autounmask-write y -av ",
                check,
            ]
            .concat();
            let shellout_result = linux::system_command(&cmdline);
            linux::exit_on_failure(&shellout_result);
        }
    }
    println!(" OK");

    if !cleanup {
        /* Now check the timestamp of the Gentoo package repo to prevent more than one sync per day
         * and if we are not too recent from the last emerge --sync, call eix-sync
         */

        chevrons::three(Colour::Green);
        println!("Initialising package database");
        portage::eix_update();

        if force {
            portage::do_eix_sync();
        } else if !portage::too_recent() {
            portage::do_eix_sync();
        }

        /* Often is it necessary to update sys-apps/portage first before updating world
         * Next we need to find out if there is an update available for portage
         * and same again for gcc
         */

        if portage::package_outdated("sys-apps/portage") {
            portage::upgrade_package("sys-apps/portage");
        }
        if portage::package_outdated("sys-devel/gcc") {
            portage::upgrade_package("sys-devel/gcc");
        }

        // Present a list of packages to be updated to the screen
        // If there are no packages pending updates, we can quit at this stage

        if !portage::eix_diff() && !force {
            process::exit(0);
        }

        chevrons::three(Colour::Green);
        println!("Fetching sources: ");
        portage::upgrade_world(Upgrade::Fetch);

        // All pre-requisites done - time for upgrade - give user a chance to quit
        if prompt::ask_user("Ready for upgrade?\t\t", PromptType::Review) {
            chevrons::three(Colour::Green);
            println!("Working... Please wait");
            portage::upgrade_world(Upgrade::Real);
        }
    }

    // Displays any messages from package installs to the user
    portage::elogv();

    // List and remove orphaned dependencies
    if portage::depclean(Upgrade::Pretend) != 0 {
        if prompt::ask_user(
            "Perform dependency cleanup as per above?",
            PromptType::Review,
        ) {
            portage::depclean(Upgrade::Real);
        }
    }

    // Check reverse dependencies
    if portage::revdep_rebuild(Upgrade::Pretend) {
        if prompt::ask_user("Perform reverse dependency rebuild?", PromptType::Review) {
            portage::revdep_rebuild(Upgrade::Real);
        }
    }

    // Check Portage sanity
    portage::eix_test_obsolete();

    // Cleanup old kernels
    if prompt::ask_user("Clean up old kernels?", PromptType::Review) {
        portage::eclean_kernel();
    }

    // Cleanup old distfiles
    if prompt::ask_user(
        "Clean up old distribution source tarballs?",
        PromptType::Review,
    ) {
        portage::eclean_distfiles();
    }

    // fstrim
    if prompt::ask_user("Reclaim free blocks?", PromptType::Review) {
        linux::call_fstrim();
    }

    chevrons::three(Colour::Green);
    println!("All done!!!");
}
