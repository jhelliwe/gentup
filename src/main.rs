// Gentoo Updater
// John Helliwell

/* This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
*/

const VERSION: &str = "0.21a";

pub mod chevrons;
pub mod linux;
pub mod portage;
pub mod prompt;
pub mod tabulate;
use crossterm::style::Color;
use std::{env, path::Path, process};

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
    // Check we are root
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
    // Parse command line arguments
    let args = env::args();
    let mut force = false;
    let mut cleanup = false;
    let mut first = true;
    let mut background = true;
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
                    -c, --cleanup    Only perform cleanup, useful if you interupped the previous run\n\
                    -f, --force      Force eix-sync, bypassing the timestamp check\n\
                    -s  --separate   Perform source fetching separately before update\n\
                    -h, --help       Display this help text, then exit\n\
                    -V, --version    Display the program version\
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
            "-s" | "--separate" => {
                background = true;
            }
            "-c" | "--cleanup" => {
                cleanup = true;
            }
            _ => {
                eprintln!("Error: usage - gentup [--help|--force|--separate|--cleanup|--version]");
                process::exit(1);
            }
        }
    }

    // Get started
    let _ignore = linux::system_command_interactive("clear", "clear");
    println!("\nWelcome to the Gentoo Updater v{}\n", VERSION);

    // Are we running on Gentoo?
    let _distro =
        linux::check_distro("Gentoo".to_string()).expect("This updater only works on Gentoo Linux");

    // Check things are installed
    chevrons::three(Color::Green);
    print!("Checking environment: ");

    // We won't get much further if eix is not installed. We must check this
    if !Path::new("/usr/bin/eix").exists() {
        let shellout_result = linux::system_command_non_interactive(
            "emerge --quiet -v app-portage/eix",
            "Installing eix",
        );
        linux::exit_on_failure(&shellout_result);
    }

    // We won't get much further if equery is not installed. We must check this too
    if !Path::new("/usr/bin/equery").exists() {
        let shellout_result = linux::system_command_non_interactive(
            "emerge --quiet -v app-portage/gentoolkit",
            "Installing gentoolkit",
        );
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
        if portage::package_is_missing(check) {
            chevrons::eerht(Color::Red);
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
                linux::system_command_interactive(&cmdline, "Installing missing package");
            linux::exit_on_failure(&shellout_result);
        }
    }
    println!(" OK");

    if !cleanup {
        // Only do update tasks if the user did not select cleanup mode
        // Make sure the eix database is up to date
        chevrons::three(Color::Green);
        println!("Initialising package database");
        portage::eix_update();

        // Check if the last resync was too recent - if not, sync the portage tree
        if force || !portage::too_recent() {
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
        prompt::ask_user("Please review", PromptType::PressCR);

        // Fetch sources
        if ! background {
            portage::upgrade_world(Upgrade::Fetch);
        }

        // Check the news - if there is news, list and read it
        chevrons::three(Color::Green);
        println!("Checking Gentoo news");
        if portage::handle_news() > 0 {
            chevrons::three(Color::Red);
            println!("Attention: You have unread news");
            prompt::ask_user("Press CR", PromptType::PressCR);
        }

        // All pre-requisites done - time for upgrade
        portage::upgrade_world(Upgrade::Real);

        // Handle updating package config files
        portage::dispatch_conf();
    }

    // Special case for cleanup mode - handle news here too.
    if cleanup {
        chevrons::three(Color::Green);
        println!("Rechecking Gentoo news");
        if portage::handle_news() > 0 {
            chevrons::three(Color::Red);
            println!("Attention: You have unread news");
            prompt::ask_user("Press CR", PromptType::PressCR);
        }
    }

    // Displays any messages from package installs to the user
    portage::elogv();

    // List and remove orphaned dependencies
    if portage::depclean(Upgrade::Pretend) != 0 {
        portage::depclean(Upgrade::Real);
    }

    // Check reverse dependencies
    if !portage::revdep_rebuild(Upgrade::Pretend)
        && prompt::ask_user("Perform reverse dependency rebuild?", PromptType::Review)
    {
        portage::revdep_rebuild(Upgrade::Real);
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

    chevrons::three(Color::Green);
    println!("All done!!!");
}
