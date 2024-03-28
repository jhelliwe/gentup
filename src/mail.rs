use crate::{linux::CouldFail, linux::OsCall, prompt, Config};
use crossterm::style::Color;
use gethostname::gethostname;
use std::{
    fs::{self, File},
    io::Write,
    process,
};

pub fn send_email(running_config: &Config, email_body: String) {
    let temp_file_name = format!("/tmp/gentup.{}.eml", process::id());
    {
        let mut temp_file = match File::create(&temp_file_name) {
            Ok(temp_file) => temp_file,
            Err(error) => {
                println!(
                    "{} Error creating email {}",
                    prompt::revchevrons(Color::Red),
                    error
                );
                process::exit(1);
            }
        };
        let _ = writeln!(temp_file, "{email_body}");

        let _ = OsCall::Quiet
            .piped(
                &["cat ", &temp_file_name].concat(),
                &["mail -s Test ", &running_config.email_address].concat(),
            )
            .exit_if_failed();
    }
    let _ = fs::remove_file(&temp_file_name);
}

pub fn test_mail(running_config: &Config) {
    send_email(
        running_config,
        format!(
            "\
    This is a test email from the Gentoo Linux Updater on {}\n\
    \n\
    Your email configuration is working correctly",
            gethostname()
                .into_string()
                .unwrap_or("localhost".to_string()),
        ),
    );
}
