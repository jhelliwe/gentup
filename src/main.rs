// Gentoo Updater version 0.03a

pub mod prompt;
pub mod check_distro;
pub mod portage;
pub mod system_command;

use std::process;
use prompt::*;
use check_distro::*;
use portage::*;
use system_command::*;

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
    
    clearscreen::clear().expect("Terminfo problem. Cannot continue");
    println!("\n\nWelcome to the Gentoo Updater\n\n");

    // Are we running on Gentoo? 
    let _distro = check_distro("Gentoo".to_string()).expect("This updater only works on Gentoo Linux");
    
    // Is the eix package installed? It is not by default
    if eix_is_missing() {
        println!("This program requires eix to be installed. Please run 'emerge -av app-portage/eix' first");
        process::exit(2);
    }

    /* Now check the timestamp of the Gentoo package repo to prevent more than one sync per day
     * and if we are not too recent from the last emerge --sync, call eix-sync
     */
    if ! too_recent() { do_eix_sync(); }
    
    /* Often is it necessary to update sys-apps/portage first before updating world
     * Next we need to find out if there is an update available for portage
     */
    if portage_outdated() { upgrade_portage() }

    // All pre-requisites done - time for upgrade - give user a chance to quit
    ask_user("\n\nReady for upgrade?\t\t", PromptType::PressCR );

    // Upgrade all installed packages 
    if ask_user("About to perform world update", PromptType::ClearScreen) { upgrade_world(); }

    // List and remove orphaned dependencies
    depclean(Upgrade::Pretend);
    if ask_user("Perform dependency cleanup as per above?", PromptType::Review ) { depclean(Upgrade::Real); }

    // Check reverse dependencies
    revdep_rebuild(Upgrade::Pretend);
    if ask_user("Perform reverse dependency rebuild?", PromptType::Review ) { revdep_rebuild(Upgrade::Real); }

    // Check Portage sanity
    eix_test_obsolete();

    // Cleanup old kernels
    if ask_user("Clean up old kernels?", PromptType::Review ) { eclean_kernel(); }

    // Cleanup old distfiles
    if ask_user("Clean up old distribution source tarballs?", PromptType::Review ) { eclean_distfiles(); }

    // fstrim
    if ask_user("Reclaim free blocks?", PromptType::Review ) { call_fstrim(); }

    println!("All done!!!");
}
