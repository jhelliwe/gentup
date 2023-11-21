// Gentoo Updater version 0.01a

use std::fs::{self,File};
use std::io::{self,BufRead,BufReader};
use std::process::{self,Command};
use filetime::FileTime;
use chrono;

enum Upgrade {
    Real,
    Pretend,
}

fn check_distro(required_distro: String) -> Result<String, String>  {
    println!("1. Checking linux distribution name");
    let os_release = File::open("/etc/os-release").expect("/etc/os-release should be readable!");
    let readbuf = BufReader::new(os_release);
    let firstline = readbuf.lines().next().expect("Could not read /etc/os-release").unwrap();
    let parts = firstline.split('=');
    let parts: Vec<&str> = parts.collect();
    let detected_distro = parts[1].to_string();
    println!("2. Running on {}", detected_distro);
    match required_distro.eq(&detected_distro) {
        true => Ok(detected_distro),
        false => Err(detected_distro),
    }
}

fn too_recent() -> bool {
    let portage_metadata = fs::metadata("/var/db/repos/gentoo/metadata/timestamp").unwrap();
    let filestamp = FileTime::from_last_modification_time(&portage_metadata).seconds() ;
    let nowutc = chrono::offset::Utc::now();
    let nowstamp = nowutc.timestamp();

    if nowstamp - filestamp < (24 * 60 * 60) {
        println!("Last emerge --sync was less than 24 hours ago. We will skip doing an eix-sync");
        return true;
    } else {
        return false;
    }
}

fn do_eix_sync() {
    check_with_user("3. Updating the Portage tree - performing an 'eix-sync'");
    let process = Command::new("eix-sync")
    .spawn()
    .expect("EIX-SYNC");
    let _output = match process.wait_with_output() {
        Ok(output)  => output,
        Err(err)    => panic!("Retrieving output error: {}", err),
    };
    check_with_user("Please verify the output of eix-sync above");
}

fn check_with_user(message: &str) {
    println!("{message}: Press Return to continue, Enter 'q' to to quit, or Ctrl-C to quit");
    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    if user_input.eq("q\n") { println!("Quitting at user request"); process::exit(0); }
}

fn portage_outdated() -> bool {
    println!("4. Checking if sys-apps/portage is up to date");
    let process = Command::new("eix")
    .arg("-u")
    .arg("sys-apps/portage")
    .output()
    .expect("EIX PORTAGE");
    let output = String::from_utf8_lossy(&process.stdout);
    if output.eq("No matches found\n") {
        println!("5. Portage is up to date");
        return false;
    }
    println!("Portage needs ugrade");
    true
}

fn upgrade_portage() {
    check_with_user("5. Performing 'emerge -1av sys-apps/portage");
        let process = Command::new("emerge")
        .arg("--quiet")
        .arg("-1av")
        .arg("portage")
        .spawn()
        .expect("EMERGE PORTAGE");
        let _output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
        };
        check_with_user("Please verify the output of emerge portage above");
}

fn upgrade_world() {
        check_with_user("6. Performing 'emerge --quiet -auNDv --with-bdeps y --complete-graph --changed-use @world");
        let process = Command::new("emerge")
        .arg("--quiet")
        .arg("-auNDv")
        .arg("--with-bdeps")
        .arg("y")
        .arg("--changed-use")
        .arg("--complete-graph")
        .arg("@world")
        .spawn()
        .expect("EMERGE WORLD");
        let _output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
        };
        check_with_user("Please verify the output of emerge world above");
}

fn depclean(run_type: Upgrade) {
    match run_type {
        Upgrade::Pretend => {
            check_with_user("7. Performing 'emerge --pretend --depclean'");
            let process = Command::new("emerge")
            .arg("-p")
            .arg("--depclean")
            .spawn()
            .expect("DEPCLEAN");
            let _output = match process.wait_with_output() {
                Ok(output)  => output,
                Err(err)    => panic!("Retrieving output error: {}", err),
            };
        check_with_user("Please verify the output of emerge --pretend --depclean above");
        },

            Upgrade::Real => {
            check_with_user("8. Performing 'emerge --depclean'");
            let process = Command::new("emerge")
            .arg("--depclean")
            .spawn()
            .expect("DEPCLEAN");
            let _output = match process.wait_with_output() {
                Ok(output)  => output,
                Err(err)    => panic!("Retrieving output error: {}", err),
            };
        check_with_user("Please verify the output of emerge --depclean above");
        }
    }
}

fn revdep_rebuild(run_type: Upgrade) {
    match run_type {
        Upgrade::Pretend => { 
            check_with_user("9. Performing 'revdep-rebuild -pv'");
            let process = Command::new("revdep-rebuild")
            .arg("-pv")
            .spawn()
            .expect("REVDEP");
            let _output = match process.wait_with_output() {
                Ok(output)  => output,
                Err(err)    => panic!("Retrieving output error: {}", err),
            };
            check_with_user("Please verify the output of revdep-rebuild -pv above");
        },
        Upgrade::Real => { 
            check_with_user("10. Performing 'revdep-rebuild'");
            let process = Command::new("revdep-rebuild")
            .spawn()
            .expect("REVDEP");
            let _output = match process.wait_with_output() {
                Ok(output)  => output,
                Err(err)    => panic!("Retrieving output error: {}", err),
            };
            check_with_user("Please verify the output of revdep-rebuild above");
        },
    }
}

fn eix_test_obsolete() {
        check_with_user("11. Performing 'eix-test-obsolete'");
        let process = Command::new("eix-test-obsolete")
        .spawn()
        .expect("OBSOLETE");
        let _output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
        };
        check_with_user("Please verify the output of eix above");
}


fn eclean_kernel() {
        check_with_user("12. Performing 'eclean-kernel -Aa'");
        let process = Command::new("eclean-kernel")
        .arg("-Aa")
        .spawn()
        .expect("KERNEL");
        let _output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
        };
        check_with_user("Please verify the output of eclean above");
}

fn eclean_distfiles() {
        check_with_user("13. Performing 'eclean -d distfiles'");
        let process = Command::new("eclean")
        .arg("-d")
        .arg("distfiles")
        .spawn()
        .expect("DISTFILES");
        let _output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
        };
        check_with_user("Please verify the output of eclean above");
}

fn call_fstrim() {
        check_with_user("14. Performing 'fstrim -av'");
        let process = Command::new("fstrim")
        .arg("-av")
        .spawn()
        .expect("FSTRIM");
        let _output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
        };
        check_with_user("Please verify the output of fstrim above");
}

fn main() {
    
    /* Check our running environment
     * Are we running on Gentoo? 
     */

    let distro = check_distro("Gentoo".to_string()).expect("This updater only works on Gentoo!");
    println!("\n\nWelcome to the {} Updater\n\n", distro);

    /* Now check the timestamp of the Gentoo package repo to prevent more than one sync per day
     * and if we are not too recent from the last emerge --sync, call eix-sync
     */

    if ! too_recent() {
     do_eix_sync();
    }
    
    /* Often is it necessary to update sys-apps/portage first before updating world
     * Next we need to find out if there is an update available for portage
     */

    if portage_outdated() {
        upgrade_portage()
    }

    // Upgrade all installed packages 
    upgrade_world();

    // List and remove orphaned dependencies
    depclean(Upgrade::Pretend);
    depclean(Upgrade::Real);

    // Check reverse dependencies
    revdep_rebuild(Upgrade::Pretend);
    revdep_rebuild(Upgrade::Real);

    // Check Portage sanity
    eix_test_obsolete();

    // Cleanup old kernels
    eclean_kernel();

    // Cleanup old distfiles
    eclean_distfiles();

    // fstrim
    call_fstrim();

    println!("All done!!!");
}
