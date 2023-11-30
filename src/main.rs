// Gentoo Updater version 0.02a

pub mod prompt;
pub mod check_distro;
pub mod portage;

use std::process::{Command};
use prompt::*;
use check_distro::*;
use portage::*;

pub enum Upgrade {
    Real,
    Pretend,
}

fn call_fstrim() {
        let process = Command::new("fstrim")
        .arg("-av")
        .spawn()
        .expect("FSTRIM");
        let _output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
        };
}

fn cls() { 
    clearscreen::clear().expect("Terminfo problem. Cannot continue");
}

fn main() {
    
    cls();
    println!("\n\nWelcome to the Gentoo Updater\n\n");

    // Are we running on Gentoo? 

    let _distro = check_distro("Gentoo".to_string()).expect("This updater only works on Gentoo Linux");

    /* Now check the timestamp of the Gentoo package repo to prevent more than one sync per day
     * and if we are not too recent from the last emerge --sync, call eix-sync
     */

    if ! too_recent() { do_eix_sync(); }
    
    /* Often is it necessary to update sys-apps/portage first before updating world
     * Next we need to find out if there is an update available for portage
     */

    if portage_outdated() { upgrade_portage() }

    // All pre-requisites done - time for upgrade - give user a chance to quit
    press_cr("\n\nReady for upgrade?\t\t");

    // Upgrade all installed packages 
    if ask_user("About to perform world update") { upgrade_world(); }

    // List and remove orphaned dependencies
    depclean(Upgrade::Pretend);
    if review_output("Perform dependency cleanup as per above?") { depclean(Upgrade::Real); }

    // Check reverse dependencies
    revdep_rebuild(Upgrade::Pretend);
    if review_output("Perform reverse dependency rebuild?") { revdep_rebuild(Upgrade::Real); }

    // Check Portage sanity
    eix_test_obsolete();

    // Cleanup old kernels
    if review_output("Clean up old kernels?") { eclean_kernel(); }

    // Cleanup old distfiles
    if review_output("Clean up old distribution source tarballs?") { eclean_distfiles(); }

    // fstrim
    if review_output("Reclaim free blocks?") { call_fstrim(); }

    println!("All done!!!");
}
