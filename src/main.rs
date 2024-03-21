// Gentoo Updater
// Written by John Helliwell
// https://github.com/jhelliwe

/* This program is free software: you can redistribute it
 * and/or modify it under the terms of the GNU General
 * Public License as published by the Free Software Foundation,
 * either version 3 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of i
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
*/

// Declare the modules used by the project
//
pub mod args;
pub mod config;
pub mod linux;
pub mod mail;
pub mod portage;
pub mod prompt;
pub mod version;

use crate::{
    args::{ArgCheck, ArgumentStruct, Search},
    config::{Config, CONFIG_FILE_PATH, PACKAGE_FILE_PATH},
    linux::CouldFail,
    portage::PackageManager,
    prompt::Prompt,
    version::VERSION,
};
use crossterm::style::Color;
use std::{env, path::Path, process};

// main is the entry point for the compiled binary executable
//
fn main() {
    //
    // Construct a Vector containing the list of valid command line options for this program
    // There is logic in ArgCheck to construct a "usage", "help", and syntax-check any passed
    // command line arguments against this Vector
    //
    let mut arg_syntax = vec![ArgumentStruct::from(
        "b",
        "background",
        "Perform source fetching in the background during update",
    )];
    arg_syntax.push(ArgumentStruct::from(
        "c",
        "cleanup",
        "Perform cleanup tasks after a successful upgrade",
    ));
    arg_syntax.push(ArgumentStruct::from(
        "f",
        "force",
        "Force package tree sync, bypassing the timestamp check",
    ));
    arg_syntax.push(ArgumentStruct::from(
        "h",
        "help",
        "Display this help text, then exit",
    ));
    arg_syntax.push(ArgumentStruct::from(
        "o",
        "optional",
        &["Install optional packages listed in ", PACKAGE_FILE_PATH].concat(),
    ));
    arg_syntax.push(ArgumentStruct::from(
        "s",
        "setup",
        "Set configuration options",
    ));
    arg_syntax.push(ArgumentStruct::from(
        "t",
        "trim",
        "Perform an fstrim after the upgrade",
    ));
    arg_syntax.push(ArgumentStruct::from(
        "u",
        "unattended",
        "Unattended upgrade - only partially implemented",
    ));
    arg_syntax.push(ArgumentStruct::from(
        "V",
        "version",
        "Display the program version",
    ));

    // There is a configuration file for this program, by default in /etc/conf.d/gentup
    // Load the saved config (or generate defaults if it doesn't exist)
    //
    let running_config = if Path::new(&CONFIG_FILE_PATH).exists() {
        Config::load()
    } else {
        Config::build_default().save()
    };

    // Parse the command line arguments supplied by the user
    // The Result is either Ok or Err to indicate if the arguments were parsable according to the
    // arg_syntax generated above
    //
    match ArgCheck::parse(arg_syntax, env::args()) {
        Err(error) => {
            // Command line arguments are incorrect - inform the user and exit
            eprintln!("{}", error);
            process::exit(1);
        }
        Ok(arguments) => {
            linux::clearscreen();

            // =============
            // PREREQUSITES
            // =============

            // Check that this is running on Gentoo. If not, exit with an error
            //
            match linux::check_distro("Gentoo") {
                Err(error) => {
                    eprintln!("{error}");
                    process::exit(1);
                }
                Ok(distro) => {
                    println!("\nWelcome to the {} Updater v{}\n", distro, VERSION);
                }
            }

            // Handle program setup
            if arguments.get("setup") {
                config::setup();
                process::exit(0);
            }

            portage::check_and_install_deps(); // This call installs any missing dependencies of this program
            
            // Check that elogv is configured - elogv collects post-installation notes for package
            // updates, so the user is notified about actions they need to take. If elogv is
            // installed but not configured, this function call will configure elogv
            //
            portage::configure_elogv();

            // If the user selected the --optional flag, check and install the optional packages.
            // This is mostly useful to get a newly installed bare-bones Gentoo install into a more
            // complete baseline state
            //
            if arguments.get("optional") {
                portage::check_and_install_optional_packages();
            }

            // Check if the last resync was too recent - if not, sync the portage tree
            // or the user can force a sync anyway by using "gentup --force"
            // The too recent logic is to avoid abusing the rsync.gentoo.org rotation which
            // asks that users do not sync more than once per day
            //
            if arguments.get("force") || !portage::too_recent() {
                portage::sync_package_tree();
            }

            // Update sys-apps/portage and sys-devel/gcc before any other packages
            // sys-apps/portage is the Gentoo package manager and portage itself advises the user to
            // update portage first
            //
            if portage::package_outdated("sys-apps/portage") {
                portage::upgrade_package("sys-apps/portage");
            }
            if portage::package_outdated("sys-devel/gcc") {
                portage::upgrade_package("sys-devel/gcc");
            }

            // Present a list of packages to be updated to the screen
            // If there are no packages pending updates, we can quit at this stage
            // unless the user specifically asked for a cleanup to be run
            //
            let pending_updates = portage::get_pending_updates(arguments.get("background"));
            if !pending_updates && (!arguments.get("cleanup") || !running_config.clean_default) {
                process::exit(0);
            }

            if !arguments.get("unattended") {
                Prompt::PressReturn.askuser("Please review");
            }

            // Check the news - if there is news, list and read it
            // TODO - make read_news email the user instead of interrupting the program flow
            //
            println!("{} Checking Gentoo news", prompt::chevrons(Color::Green));
            if portage::read_news() > 0 {
                Prompt::PressReturn.askuser("Press CR");
            }

            // ==================
            // FULL SYSTEM UPDATE
            // ==================

            if pending_updates {
                let _ = PackageManager::NoDryRun
                    .update_all_packages()
                    .exit_if_failed();
            }

            // =================
            // POST_UPDATE TASKS
            // =================

            portage::update_config_files(); // Handle updating package config files
            portage::elog_viewer(); // Displays any messages from package installs to the user

            // =======
            // CLEANUP
            // =======

            // List and remove orphaned dependencies.
            // One important point, we force a cleanup if there are old kernel packages to remove
            // otherwise /boot will become too full and cause issues with an unbootable system with
            // a /boot 100% full with a truncated initrd file
            //
            let (orphans, kernels) = PackageManager::DryRun.depclean(); // DryRun mode only lists orphaned deps
            if orphans > 0 {
                // To prevent the issue of depclean removing the currently running kernel immediately after a kernel upgrade
                // check to see if the running kernel will be depcleaned
                //
                if kernels.contains(&linux::running_kernel()) {
                    if arguments.get("cleanup") || running_config.clean_default {
                        PackageManager::PreserveKernel.depclean(); // depcleans everything excluding old kernel packages
                    }
                    println!(
                        "{} Preserving currently running kernel. Skipping cleanup",
                        prompt::chevrons(Color::Green)
                    );
                    println!("{} All done!!!", prompt::chevrons(Color::Green));
                    process::exit(0);
                } else if (arguments.get("cleanup") || running_config.clean_default)
                    || kernels.ne("")
                {
                    PackageManager::AllPackages.depclean(); // depcleans everything
                }
            }

            // Check for broken Reverse dependencies
            //
            if (arguments.get("cleanup") || running_config.clean_default) || kernels.ne("") {
                if !PackageManager::DryRun.revdep_rebuild() {
                    PackageManager::NoDryRun.revdep_rebuild();
                }
                portage::find_obsolete_configs(); // Find any obsolete portage configurations from removed packages
                portage::clean_distfiles(); // Cleanup old distfiles otherwise these will grow indefinitely
                portage::clean_old_kernels(); // Cleanup unused kernels from /usr/src, /boot, /lib/modules and the grub config

                if arguments.get("trim") || running_config.trim_default {
                    // A full update creates so many GB of temp files it warrants a trim, but only
                    // if the user specifies --trim on the command line
                    linux::call_fstrim();
                }
            }
            println!("{} All done!!!", prompt::chevrons(Color::Green));
        }
    }
}
