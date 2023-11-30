use std::fs;
use filetime::FileTime;
use std::process::{Command};
use crate::prompt::*;

pub fn too_recent() -> bool {
    let portage_metadata = fs::metadata("/var/db/repos/gentoo/metadata/timestamp").unwrap();
    let filestamp = FileTime::from_last_modification_time(&portage_metadata).seconds() ;
    let nowutc = chrono::offset::Utc::now();
    let nowstamp = nowutc.timestamp();

    if nowstamp - filestamp < (24 * 60 * 60) {
        println!("Checking timestamp of last sync\t\t Skipping sync phase");
        return true;
    } else {
        return false;
    }
}

pub fn do_eix_sync() {
    let process = Command::new("eix-sync")
    .arg("-q")
    .spawn()
    .expect("EIX-SYNC");
    let _output = match process.wait_with_output() {
        Ok(output)  => output,
        Err(err)    => panic!("Retrieving output error: {}", err),
    };
    press_cr("Please verify the output of eix-sync above");
}

pub fn portage_outdated() -> bool {
    print!("Checking portage version \t\t");
    let process = Command::new("eix")
    .arg("-u")
    .arg("sys-apps/portage")
    .output()
    .expect("EIX PORTAGE");
    let output = String::from_utf8_lossy(&process.stdout);
    if output.eq("No matches found\n") {
        println!(" sys-apps/portage is up to date");
        return false;
    }
    println!("Portage needs ugrade");
    true
}

pub fn upgrade_portage() {
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
}

pub fn upgrade_world() {
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
}

pub fn depclean(run_type: crate::Upgrade) {
    match run_type {
        crate::Upgrade::Pretend => {
            println!("Performing dependency check... Please wait");
            let process = Command::new("emerge")
            .arg("-p")
            .arg("--depclean")
            .spawn()
            .expect("DEPCLEAN");
            let _output = match process.wait_with_output() {
                Ok(output)  => output,
                Err(err)    => panic!("Retrieving output error: {}", err),
            };
        },

            crate::Upgrade::Real => {
            let process = Command::new("emerge")
            .arg("--depclean")
            .spawn()
            .expect("DEPCLEAN");
            let _output = match process.wait_with_output() {
                Ok(output)  => output,
                Err(err)    => panic!("Retrieving output error: {}", err),
            };
        press_cr("Please verify the output of emerge --depclean above");
        }
    }
}

pub fn revdep_rebuild(run_type: crate::Upgrade) {
    match run_type {
        crate::Upgrade::Pretend => { 
            println!("Performing reverse dependency check... Please wait");
            let process = Command::new("revdep-rebuild")
            .arg("-pq")
            .spawn()
            .expect("REVDEP");
            let _output = match process.wait_with_output() {
                Ok(output)  => output,
                Err(err)    => panic!("Retrieving output error: {}", err),
            };
        },
        crate::Upgrade::Real => { 
            let process = Command::new("revdep-rebuild")
            .spawn()
            .expect("REVDEP");
            let _output = match process.wait_with_output() {
                Ok(output)  => output,
                Err(err)    => panic!("Retrieving output error: {}", err),
            };
            press_cr("Please verify the output of revdep-rebuild above");
        },
    }
}

pub fn eix_test_obsolete() {
        println!("Performing portage hygiene tests");
        let process = Command::new("eix-test-obsolete")
        .spawn()
        .expect("OBSOLETE");
        let _output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
        };
}


pub fn eclean_kernel() {
        let process = Command::new("eclean-kernel")
        .arg("-Aa")
        .spawn()
        .expect("KERNEL");
        let _output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
        };
}

pub fn eclean_distfiles() {
        let process = Command::new("eclean")
        .arg("-d")
        .arg("distfiles")
        .spawn()
        .expect("DISTFILES");
        let _output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
        };
}
