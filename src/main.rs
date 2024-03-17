// Gentoo Updater
// Written by John Helliwell
// https://github.com/jhelliwe

const VERSION: &str = "0.41a";

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
pub mod args; // Deals with command line arguments
pub mod linux; // Interacts with the operating system
pub mod portage; // Interacts with the Portage package manager
pub mod prompt; // Asks the user permission to continue
use crate::{args::GentupArgs, linux::CouldFail, portage::Emerge, prompt::Prompt};
use crossterm::{
    cursor, execute,
    style::Color,
    terminal::{self, ClearType},
};
use std::{env, io, process};

// main is the entry point for the compiled binary executable
fn main() {
    // First, parse the command line arguments
    match GentupArgs::parse(env::args()) {
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

            // Are we running on Gentoo? if not, exit the program
            match linux::check_distro("Gentoo") {
                Err(error) => {
                    eprintln!("{error}");
                    process::exit(1);
                }
                Ok(distro) => {
                    println!("\nWelcome to the {} Updater v{}\n", distro, VERSION);
                }
            }

            // This call installs required packages which are a direct dependency of this updater,
            portage::check_and_install_deps();
            if arguments.optional {
                portage::check_and_install_optional_packages();
            }

            // Check that elogv is configured - elogv collects post-installation notes for package
            // updates, so the user is notified about actions they need to take. If elogv is
            // installed but not configured, this function call will configure elogv for us
            portage::configure_elogv();

            // Check if the last resync was too recent - if not, sync the portage tree
            // or the user can force a sync anyway by using "gentup --force"
            // The too recent logic is to avoid abusing the rsync.gentoo.org rotation which
            // asks that users do not sync more than once per day
            if arguments.force || !portage::too_recent() {
                portage::sync_package_tree();
            }

            /* It necessary to update the package manager (sys-apps/portage) first before updating
             * the entire system and the same again for gcc since this is a source based
             * distribution!
             */

            if portage::package_outdated("sys-apps/portage") {
                portage::upgrade_package("sys-apps/portage");
            }

            if portage::package_outdated("sys-devel/gcc") {
                portage::upgrade_package("sys-devel/gcc");
            }

            // Present a list of packages to be updated to the screen
            // If there are no packages pending updates, we can quit at this stage
            let pending = portage::get_pending_updates(arguments.background_fetch);
            if !pending && !arguments.force && !arguments.cleanup {
                process::exit(0);
            }
            if !arguments.unattended {
                Prompt::ReturnOrQuit.askuser("Please review");
            }

            // Check the news - if there is news, list and read it
            println!("{} Checking Gentoo news", prompt::chevrons(Color::Green));
            if portage::read_news() > 0 {
                Prompt::ReturnOrQuit.askuser("Press CR"); // Eventually read_news will email the user instead of pronpting
            }

            // All pre-requisites done - time for upgrade
            if pending {
                let _ = Emerge::Real.update_world().exit_if_failed();
            }

            // Handle updating package config files
            portage::update_config_files();

            // Displays any messages from package installs to the user
            portage::elog_viewer();

            // List and remove orphaned dependencies.
            // One important point, we force a cleanup if there are old kernel packages to remove
            // otherwise /boot will become too full
            let (orphans, kernels) = Emerge::Pretend.depclean(); // Pretend mode only lists orphaned deps
            if orphans > 0 {
                // To prevent the issue of depclean removing the currently running kernel immedately after a kernel upgrade
                // check to see if the running kernel will be depcleaned
                if kernels.contains(&linux::running_kernel()) {
                    if arguments.cleanup {
                        Emerge::RealExcludeKernels.depclean(); // depcleans everything excluding old kernel packages
                    }
                    println!(
                        "{} Preserving currently running kernel. Skipping cleanup",
                        prompt::chevrons(Color::Green)
                    );
                    println!("{} All done!!!", prompt::chevrons(Color::Green));
                    process::exit(0);
                } else if arguments.cleanup || kernels.ne("") {
                    Emerge::RealIncludeKernels.depclean(); // depcleans everything
                }
            }

            if arguments.cleanup || kernels.ne("") {
                // Check and rebuild any broken reverse dependencies
                if !Emerge::Pretend.revdep_rebuild() {
                    Emerge::Real.revdep_rebuild();
                }

                // Check the sanity of the /etc/portage configuration files. These can become complex
                // configurations over time, and entries in this directory can break things when
                // packages that required these configs are removed. This feature makes eix worth its
                // weight in gold. Good portage hygiene prevents problems further down the line
                portage::eix_test_obsolete();

                // Cleanup old distfiles
                portage::eclean_distfiles();

                // Cleanup unused kernels from /usr/src, /boot, /lib/modules and the grub config
                portage::eclean_kernel();

                // fstrim - if this is an SSD or thinly provisioned filesystem, send DISCARDs so that
                // the backing store can recover any freed-up blocks. I do this because a full
                // update creates so many GB of temp files it warrants a trim. If the user selects
                // the trim option, they are responsible for ensuring the device supports trim/discard
                if arguments.trim {
                    linux::call_fstrim();
                }
            }
            println!("{} All done!!!", prompt::chevrons(Color::Green));
        }
    }
}
