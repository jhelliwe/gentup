// Gentoo Updater
// Written by John Helliwell
// https://github.com/jhelliwe

const VERSION: &str = "0.43a";

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
pub mod args; // Deals with command line arguments
pub mod linux; // Interacts with the operating system
pub mod portage; // Interacts with the Portage package manager
pub mod prompt; // Asks the user permission to continue

use crate::{
    args::{ArgCheck, Search},
    linux::CouldFail,
    portage::PackageManager,
    prompt::Prompt,
};
use crossterm::{
    cursor, execute,
    style::Color,
    terminal::{self, ClearType},
};
use std::{env, io, process};

// main is the entry point for the compiled binary executable
//
fn main() {
    // First, parse the command line arguments
    //
    match ArgCheck::parse(env::args()) {
        Err(error) => {
            // Command line arguments are incorrect - inform the user and exit
            eprintln!("{}", error);
            process::exit(1);
        }
        Ok(arguments) => {
            // Clear screen
            let _ = execute!(
                io::stdout(),
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            );

            // *************
            // PREREQUSITES
            // *************

            // Are we running on Gentoo? if not, exit the program
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

            portage::check_and_install_deps(); // This call installs any missing dependencies of this program

            // If the user selected the --optional flag, check and install the optional packages.
            // This is mostly useful to get a newly installed bare-bones Gentoo install into a more
            // complete baseline state
            //
            if arguments.getflag("optional") {
                portage::check_and_install_optional_packages();
            }

            // Check that elogv is configured - elogv collects post-installation notes for package
            // updates, so the user is notified about actions they need to take. If elogv is
            // installed but not configured, this function call will configure elogv for us
            //
            portage::configure_elogv();

            // Check if the last resync was too recent - if not, sync the portage tree
            // or the user can force a sync anyway by using "gentup --force"
            // The too recent logic is to avoid abusing the rsync.gentoo.org rotation which
            // asks that users do not sync more than once per day
            //
            if arguments.getflag("force") || !portage::too_recent() {
                portage::sync_package_tree();
            }

            // Update sys-apps/portage and sys-devel/gcc before any other packages
            //
            if portage::package_outdated("sys-apps/portage") {
                portage::upgrade_package("sys-apps/portage");
            }
            if portage::package_outdated("sys-devel/gcc") {
                portage::upgrade_package("sys-devel/gcc");
            }

            // Present a list of packages to be updated to the screen
            // If there are no packages pending updates, we can quit at this stage
            //
            let pending = portage::get_pending_updates(arguments.getflag("background"));
            if !pending && !arguments.getflag("cleanup") {
                process::exit(0);
            }
            if !arguments.getflag("unattended") {
                Prompt::PressReturn.askuser("Please review");
            }

            // Check the news - if there is news, list and read it
            //
            println!("{} Checking Gentoo news", prompt::chevrons(Color::Green));
            if portage::read_news() > 0 {
                Prompt::PressReturn.askuser("Press CR"); // Eventually read_news will email the user instead of pronpting
                                                         // in order to reach the fully automated
                                                         // milestone of this project
            }

            // ==================
            // FULL SYSTEM UPDATE
            // ==================

            if pending {
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
            // otherwise /boot will become too full. No one enjoys having to recover a non-bootable
            // system caused by a truncated initrd and few people have a massive /boot filesystem
            //
            let (orphans, kernels) = PackageManager::DryRun.depclean(); // DryRun mode only lists orphaned deps
            if orphans > 0 {
                // To prevent the issue of depclean removing the currently running kernel immediately after a kernel upgrade
                // check to see if the running kernel will be depcleaned
                //
                if kernels.contains(&linux::running_kernel()) {
                    if arguments.getflag("cleanup") {
                        PackageManager::PreserveKernel.depclean(); // depcleans everything excluding old kernel packages
                    }
                    println!(
                        "{} Preserving currently running kernel. Skipping cleanup",
                        prompt::chevrons(Color::Green)
                    );
                    println!("{} All done!!!", prompt::chevrons(Color::Green));
                    process::exit(0);
                } else if arguments.getflag("cleanup") || kernels.ne("") {
                    PackageManager::AllPackages.depclean(); // depcleans everything
                }
            }

            // Check for broken Reverse dependencies
            //
            if arguments.getflag("cleanup") || kernels.ne("") {
                if !PackageManager::DryRun.revdep_rebuild() {
                    PackageManager::NoDryRun.revdep_rebuild();
                }

                portage::eix_test_obsolete(); // Find any obsolete portage configurations from removed packages

                portage::eclean_distfiles(); // Cleanup old distfiles

                portage::eclean_kernel(); // Cleanup unused kernels from /usr/src, /boot, /lib/modules and the grub config

                if arguments.getflag("trim") {
                    // A full update creates so many GB of temp files it warrants a trim
                    linux::call_fstrim();
                }
            }
            println!("{} All done!!!", prompt::chevrons(Color::Green));
        }
    }
}
