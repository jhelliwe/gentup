// Gentoo Updater version 0.07a
const VERSION: &str = "0.07a";

pub mod linux;
pub mod portage;
pub mod prompt;
use std::path::Path;
use std::process;

pub enum Upgrade {
    Real,
    Pretend,
}

#[derive(PartialEq)]
pub enum PromptType {
    ClearScreen,
    Review,
    PressCR,
}

fn main() {
    let _ = clearscreen::clear(); 
    println!("\n\nWelcome to the Gentoo Updater v{}\n\n", VERSION);

    // Are we running on Gentoo?
    let _distro =
        linux::check_distro("Gentoo".to_string()).expect("This updater only works on Gentoo Linux");

    println!(">>> Checking environment");
    // We won't get much further if eix is not installed. We must check this
    if !Path::new("/usr/bin/eix").exists() {
        let mut shellout_result = linux::system_command("emerge --quiet -v app-portage-eix");
        linux::exit_on_failure(&shellout_result);
        shellout_result = linux::system_command("eix-update");
        linux::exit_on_failure(&shellout_result);
    }

    // Check some required (by me) packages are installed. Useful for a just-installed Gentoo
    let packages_to_check = [
        "app-portage/eix",
        "app-portage/gentoolkit",
        "app-portage/pfl",
        "app-portage/ufed",
        "app-admin/eclean-kernel",
        "net-dns/bind-tools",
    ];
    for check in packages_to_check {
        if portage::package_is_missing(&check) {
            println!(
                "This program requires {} to be installed. Installing...",
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

    /* Now check the timestamp of the Gentoo package repo to prevent more than one sync per day
     * and if we are not too recent from the last emerge --sync, call eix-sync
     */

    if !portage::too_recent() {
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
    
    if !portage::eix_diff() {
        process::exit(0);
    }

    // All pre-requisites done - time for upgrade - give user a chance to quit
    if prompt::ask_user("\n\nReady for upgrade?\t\t", PromptType::Review) {
        portage::upgrade_world();
    }

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

    println!(">>> All done!!!");
}
