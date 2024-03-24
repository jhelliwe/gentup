//
// TODO add a test() so the user can send a test mail once setup is complete
//
use crate::{prompt, Config, Prompt};
use crossterm::style::Color;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use psph::Password;
use std::{fs, fs::File, io::Write, os::unix::fs::PermissionsExt, path::Path, process};

static PASSPHRASE_PATH: &str = "/etc/conf.d/gentup.key";

// Reads the encryption key from the filesystem. If the file does not exist, generate a new key
// always enforce read-only for root, no access permissions for anyone else
//
pub fn get_passphrase() -> String {
    let passphrase = if !Path::new(&PASSPHRASE_PATH).exists() {
        generate_passphrase()
    } else {
        let wrapped_key = fs::read_to_string(PASSPHRASE_PATH);
        match wrapped_key {
            Ok(passphrase) => passphrase,
            Err(error) => {
                println!(
                    "{} Could not read key {} - {}",
                    prompt::revchevrons(Color::Red),
                    &PASSPHRASE_PATH,
                    error
                );
                process::exit(1);
            }
        }
    };
    let metaresult = fs::metadata(PASSPHRASE_PATH);
    if let Ok(metadata) = metaresult {
        let mut perms = metadata.permissions();
        perms.set_mode(0o400);
        let _ = fs::set_permissions(PASSPHRASE_PATH, perms);
    }
    passphrase
}

// Randomly generates a new encryption key into the configuration file
//
pub fn generate_passphrase() -> String {
    let newpass = Password::new(1, " ", vec!['!', '$', '@']);
    let passphrase = newpass.passphrase();
    let filepath = Path::new(&PASSPHRASE_PATH);
    let display = filepath.display();
    let mut handle = match File::create(filepath) {
        Err(error) => {
            println!(
                "{} Could not create {} - {}",
                prompt::revchevrons(Color::Red),
                display,
                error
            );
            process::exit(1);
        }
        Ok(created_ok) => created_ok,
    };
    let _ = writeln!(handle, "{passphrase}");
    passphrase.to_string()
}

// Prompts the user to enter a new email password, retrieves the encryption key, and encrypts the
// password
//
pub fn add_secret(mut running_config: Config) -> Config {
    println!(
        "{} Mail setup for {} via {}",
        prompt::chevrons(Color::Green),
        running_config.email_address,
        running_config.mta_fqdn
    );
    let passphrase = get_passphrase();
    running_config.passwd = Prompt::Options
        .askuser("Please enter your SMTP password ")
        .unwrap_or("".to_string());
    let mcrypt = new_magic_crypt!(passphrase, 256);
    running_config.encrypted_passwd = mcrypt.encrypt_str_to_base64(&running_config.passwd);
    running_config.passwd = "encrypted".to_string();
    running_config
}

// Retrieves the encryption key and decrypts the user's email password
//
pub fn get_secret(mut running_config: Config) -> Config {
    let passphrase = get_passphrase();
    let mcrypt = new_magic_crypt!(passphrase, 256);
    let wrapped_secret = mcrypt.decrypt_base64_to_string(&running_config.encrypted_passwd);
    if let Ok(secret) = wrapped_secret {
        running_config.passwd = secret;
    } else {
        println!(
            "{} Could not decrypt the password. Please setup your password correctly",
            prompt::chevrons(Color::Red)
        );
        process::exit(1);
    }
    running_config
}
