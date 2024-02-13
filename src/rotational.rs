use crate::linux;
use crate::rotational;
use crate::CmdVerbose::Quiet;
use std::fs;

pub fn getdev_rootfs() -> String {
    let mut rootfsdev = "None".to_string();
    let procmounts = fs::read_to_string("/proc/mounts");
    match procmounts {
        Ok(contents) => {
            for eachline in contents.lines() {
                if eachline.contains(" / ") {
                    let rootfsvec: Vec<&str> = eachline.split_whitespace().collect();
                    rootfsdev = rootfsvec[0].to_string();
                    break;
                }
            }
            rootfsdev.to_string()
        }
        Err(error) => {
            eprintln!("Error {}", error);
            "None".to_string()
        }
    }
}

pub fn stripchar(devicename: String) -> String {
    return devicename.chars().filter(|c| c.is_numeric()).collect();
}

pub fn major_device_number(devnode: String) -> String {
    let shellout_result = linux::system_command(&["ls -l ", &devnode].concat(), "", Quiet);
    linux::exit_on_failure(&shellout_result);
    if let (Ok(output), _) = shellout_result {
        let lsvec: Vec<&str> = output.split_whitespace().collect();
        let maj = lsvec[4];
        let newmaj = stripchar(maj.to_string());
        return newmaj;
    }
    "0".to_string()
}

pub fn syspath(major: String) -> String {
    ["/sys/dev/block/", &major, ":0/queue/rotational"].concat()
}

pub fn is_rotational() -> i32 {
    let return_value: i32 = 1;
    let device_name = rotational::getdev_rootfs();
    let device_major = rotational::major_device_number(device_name);
    let sys = rotational::syspath(device_major);
    let result = fs::read_to_string(sys);
    if let Ok(hdd) = result {
        return hdd.trim().parse::<i32>().unwrap();
    }
    return_value
}
