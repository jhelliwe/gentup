// Gentoo Updater
// Written by John Helliwell
// https://github.com/jhelliwe

const VERSION: &str = "0.28a";

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

pub mod args; // Deals with command line arguments
pub mod chevrons; // Draws coloured >>> prompts
pub mod linux; // Interacts with the operating system
pub mod portage; // Interacts with the Portage package manager
pub mod prompt; // Asks the user permission to continue
pub mod rotational; // Finds out if the root filesystem is on a rotational hard disk or SSD
pub mod tabulate; // Pretty prints a list of packages in a tabulated fashion
pub mod typedefs; // Common type definitions // Get device node information

use crate::typedefs::*;
use crossterm::{
    cursor, execute,
    style::Color,
    terminal::{self, ClearType},
};
use std::{env, io, process};

fn main() {
    // Call a functon to parse the command line arguments,
    // returning the Some veriant (and the args in a struct)
    // if the arguments were parsable, or the None variant if
    // the arguments were invalid
    let optarg = args::cmdlinargs(env::args());

    match optarg {
        Some(arguments) => {
            // Clear screen
            let _ignore = execute!(
                io::stdout(),
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            );

            // Print a welcome banner
            println!("\nWelcome to the Gentoo Updater v{}\n", VERSION);

            // Are we running on Gentoo? if not, panic the program
            let _distro = linux::check_distro("Gentoo".to_string())
                .expect("This updater only works on Gentoo Linux");

            // This call installs commonly required packages. Some are a direct dependency of this updater,
            // like eix, elogv, portage-tools etc
            // but some are just useful packages I usually install on a brand new Gentoo install
            portage::check_and_install_deps();

            // Check that elogv is configured - elogv collects post-installation notes for package
            // updates, so the user is notified about actions they need to take. If elogv is
            // installed but not configured, this function call will configure elogv for us
            portage::elog_make_conf();

            // Only do update tasks if the user did not select cleanup mode
            if !arguments.cleanup {
                // Check if the last resync was too recent - if not, sync the portage tree
                // or the user can force a sync anyway by using "gentup --force"
                // The too recent logic is to avoid abusing the rsync.gentoo.org rotation which
                // asks that users do not sync more than once per day
                if arguments.force || !portage::too_recent() {
                    portage::do_eix_sync();
                }

                /* Often is it necessary to update sys-apps/portage first before updating world
                 * and same again for gcc.
                 */
                if portage::package_outdated("sys-apps/portage") {
                    portage::upgrade_package("sys-apps/portage");
                }
                if portage::package_outdated("sys-devel/gcc") {
                    portage::upgrade_package("sys-devel/gcc");
                }

                // Present a list of packages to be updated to the screen
                // If there are no packages pending updates, we can quit at this stage
                if !portage::portage_diff() && !arguments.force {
                    process::exit(0);
                }
                prompt::ask_user("Please review", PromptType::PressCR);

                // Fetch sources as per command line argument
                // A normal update fetches the sources in the background anyway
                // but sometimes it is useful to fetch separately, for example
                // with restricted Internet access
                if arguments.separate {
                    portage::upgrade_world(Upgrade::Fetch);
                }

                // Check the news - if there is news, list and read it
                chevrons::three(Color::Green);
                println!("Checking Gentoo news");
                if portage::handle_news() > 0 {
                    prompt::ask_user("Press CR", PromptType::PressCR);
                }

                // All pre-requisites done - time for upgrade
                //if prompt::ask_user("Ready for upgrade?", PromptType::Review) {
                    portage::upgrade_world(Upgrade::Real);
                //}

                // Handle updating package config files
                portage::dispatch_conf();
            }

            // Displays any messages from package installs to the user
            portage::elogv();

            // List and remove orphaned dependencies. The depclean function returns a tuple
            // describing how many packages are orphaned in total, and also how many of those
            // packages are related to the kernel. If we have just installed a new kernel, the
            // running kernel is an immediate target for cleaning by depclean, which doesn't make
            // sense if it is still the running kernel, so there is logic in here to prevent that.
            // After all, if the kernel we have just installed fails to boot, we need to leave the
            // previous one around for recovery.
            let (orphans, kernels) = portage::depclean(DepClean::KernelPretend); // Pretend mode only lists orphaned deps
            if !arguments.cleanup && kernels > 0 {
                chevrons::three(Color::Blue);
                println!("Upgrade complete. You should reboot into your new kernel and rerun this utility with the --cleanup flag");
                // There are outstanding tasks, blocked due to the pending reboot into the new
                // kernel. Exit here to allow the user to reboot
                process::exit(0);
            }
            if orphans > 0 {
                // We only depclean kernel packages in cleanup mode - This is to prevent the issue of
                // depclean removing the currently running kernel immedately after a kernel upgrade
                if arguments.cleanup && kernels > 0 {
                    portage::depclean(DepClean::Kernel); // depcleans everything including old kernel packages
                } else {
                    portage::depclean(DepClean::Real); // depcleans everything excluding old kernel packages
                }
            }

            // Check reverse dependencies
            if !portage::revdep_rebuild(RevDep::Pretend)
                && prompt::ask_user("Perform reverse dependency rebuild?", PromptType::Review)
            {
                portage::revdep_rebuild(RevDep::Real);
            }

            // Check the sanity of the /etc/portage configuration files. These can become complex
            // configurations over time, and entries in this directory can break things when
            // packages that required these configs are removed. This feature makes eix worth its
            // weight in gold. Good portage hygiene prevents problems further down the line
            portage::eix_test_obsolete();

            // Cleanup old distfiles
            if prompt::ask_user(
                "Clean up old distribution source tarballs?",
                PromptType::Review,
            ) {
                portage::eclean_distfiles();
            }

            // Cleanup unused kernels from /boot, /lib/modules and the grub config
            if prompt::ask_user("Clean up old kernels?", PromptType::Review) {
                portage::eclean_kernel();
            }

            // fstrim - if this is an SSD or thinly provisioned filesystem, send DISCARDs so that
            // the backing store can recover any freed-up blocks
            let ssd = rotational::is_rotational() == 0;
            if ssd && prompt::ask_user("Reclaim free blocks?", PromptType::Review) {
                linux::call_fstrim();
            }

            chevrons::three(Color::Green);
            println!("All done!!!");
        }
        None => {
            // Command line arguments are incorrect, so exit
            // (an error has already been displayed to the user)
            process::exit(1);
        }
    }
}
