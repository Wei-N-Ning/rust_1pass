use anyhow::anyhow;
use rpassword::prompt_password_stdout;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

use crate::session::types::*;

pub fn sign_in_shorthand(conf: &SessionConfig) -> anyhow::Result<Session> {
    let mut proc = Command::new(&conf.bin_filename)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("signin")
        .arg("-r")
        .arg(&conf.shorthand)
        .spawn()?;
    let mut stdin = proc.stdin.take().ok_or(anyhow!(""))?;
    let mut stdout = proc.stdout.take().ok_or(anyhow!(""))?;
    let mut out_str = "".to_string();
    write!(
        stdin,
        "{}",
        prompt_password_stdout(&format!(
            "Your 1Password master password for shorthand({}):",
            &conf.shorthand
        ))?
    )?;
    drop(stdin);
    stdout.read_to_string(&mut out_str)?;
    drop(stdout);
    Ok(Session {
        bin_filename: conf.bin_filename.clone(),
        shorthand: conf.shorthand.clone(),
        session_code: out_str.trim().to_string(),
    })
}
