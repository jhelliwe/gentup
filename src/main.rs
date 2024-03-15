// Gentoo Updater
// Written by John Helliwell
// https://github.com/jhelliwe

const VERSION: &str = "0.40a";

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

            // Only do update tasks if the user did not select cleanup mode
            if !arguments.cleanup {
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
                if !arguments.force && !portage::get_pending_updates(arguments.background_fetch) {
                    process::exit(0);
                }
                Prompt::ReturnOrQuit.askuser("Please review");

                // Check the news - if there is news, list and read it
                println!("{} Checking Gentoo news", prompt::chevrons(Color::Green));
                if portage::read_news() > 0 {
                    Prompt::ReturnOrQuit.askuser("Press CR");
                }

                // All pre-requisites done - time for upgrade
                let _ = Emerge::Real // Really update, as opposed to run in Pretend mode
                    .update_world() // Update the world set, which is all packages
                    .exit_if_failed(); // and if the update fails, exit the entire program

                // Handle updating package config files
                portage::update_config_files();
            }

            // Displays any messages from package installs to the user
            portage::elog_viewer();

            // List and remove orphaned dependencies. The depclean function returns a tuple
            // describing how many packages are orphaned in total, and also how many of those
            // packages are related to the kernel. If we have just installed a new kernel, the
            // running kernel is an immediate target for cleaning by depclean, which doesn't make
            // sense if it is still the running kernel, so there is logic in here to prevent that.
            // After all, if the kernel we have just installed fails to boot, we need to leave the
            // previous one around for recovery.
            let (orphans, kernels) = Emerge::Pretend.depclean(); // Pretend mode only lists orphaned deps
            if orphans > 0 {
                // To prevent the issue of depclean removing the currently running kernel immedately after a kernel upgrade
                // check to see if the running kernel will be depcleaned
                if kernels.contains(&linux::running_kernel()) {
                    println!(
                        "{} Preserving currently running kernel",
                        prompt::revchevrons(Color::Yellow)
                    );
                    Emerge::RealExcludeKernels.depclean(); // depcleans everything excluding old kernel packages
                } else {
                    Emerge::RealIncludeKernels.depclean(); // depcleans everything
                }
            }

            // Check and rebuild any broken reverse dependencies
            if !Emerge::Pretend.revdep_rebuild()
                && Prompt::SkipReturnQuit.askuser("Perform reverse dependency rebuild?")
            {
                Emerge::Real.revdep_rebuild();
            }

            // Check the sanity of the /etc/portage configuration files. These can become complex
            // configurations over time, and entries in this directory can break things when
            // packages that required these configs are removed. This feature makes eix worth its
            // weight in gold. Good portage hygiene prevents problems further down the line
            portage::eix_test_obsolete();

            // Cleanup old distfiles
            if Prompt::SkipReturnQuit.askuser("Clean up old distribution source tarballs?") {
                portage::eclean_distfiles();
            }

            // Cleanup unused kernels from /usr/src, /boot, /lib/modules and the grub config
            if Prompt::SkipReturnQuit.askuser("Clean up old kernels?") {
                portage::eclean_kernel();
            }

            // fstrim - if this is an SSD or thinly provisioned filesystem, send DISCARDs so that
            // the backing store can recover any freed-up blocks
            if !arguments.notrim && linux::is_rotational() == 0 {
                linux::call_fstrim();
            }

            println!("{} All done!!!", prompt::chevrons(Color::Green));
        }
    }
}
