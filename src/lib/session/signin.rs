use crate::ReleaseNoteUrl;
use anyhow::anyhow;
use rpassword::prompt_password_stdout;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

use crate::session::types::*;

/// list all the accounts configured in the host system; only work with 1password cli 1.x
pub fn local_accounts_v1(conf: &SessionConfig) -> anyhow::Result<Vec<Account>> {
    let mut proc = Command::new(&conf.bin_filename)
        .stdout(Stdio::piped())
        .arg("signin")
        .arg("-l")
        .spawn()?;
    let mut stdout = proc
        .stdout
        .take()
        .ok_or_else(|| anyhow!("local accounts: fail to take stdout"))?;
    let mut out_str = String::with_capacity(1024);
    stdout.read_to_string(&mut out_str)?;
    drop(stdout);
    Ok(Account::from_descriptions(&out_str))
}

/// list all the accounts configured in the host system; only work with 1password cli 2.x
pub fn local_accounts_v2(conf: &SessionConfig) -> anyhow::Result<Vec<Account>> {
    let mut proc = Command::new(&conf.bin_filename)
        .stdout(Stdio::piped())
        .arg("account")
        .arg("list")
        .spawn()?;
    let mut stdout = proc
        .stdout
        .take()
        .ok_or_else(|| anyhow!("local accounts: fail to take stdout"))?;
    let mut out_str = String::with_capacity(1024);
    stdout.read_to_string(&mut out_str)?;
    drop(stdout);
    Ok(Account::from_descriptions(&out_str))
}

/// this signin function works with 1password cli 1.x
pub fn sign_in_shorthand_v1(conf: &SessionConfig) -> anyhow::Result<Session> {
    let mut proc = Command::new(&conf.bin_filename)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("signin")
        .arg("-r")
        .arg(&conf.shorthand)
        .spawn()?;
    let mut stdin = proc
        .stdin
        .take()
        .ok_or_else(|| anyhow!("signin: fail to take stdin"))?;
    let mut stdout = proc
        .stdout
        .take()
        .ok_or_else(|| anyhow!("signin: fail to take stdout"))?;
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
        session_code: SessionCode::V1PlainString(out_str.trim().to_string()),
        major_version: ReleaseNoteUrl::V1,
    })
}

/// this signin function works with 1password cli 2.x
pub fn sign_in_shorthand_v2(conf: &SessionConfig) -> anyhow::Result<Session> {
    let mut proc = Command::new(&conf.bin_filename)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("signin")
        .arg("-f")
        .arg("--account")
        .arg(&conf.shorthand)
        .spawn()?;
    let mut stdin = proc
        .stdin
        .take()
        .ok_or_else(|| anyhow!("signin: fail to take stdin"))?;
    let mut stdout = proc
        .stdout
        .take()
        .ok_or_else(|| anyhow!("signin: fail to take stdout"))?;
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
    let first = out_str.split('#').into_iter().next().unwrap();
    let kv = first.split("export ").into_iter().last().unwrap().trim();
    let segments: Vec<&str> = kv.split('=').collect();
    let key = segments[0];
    let value = segments[1].trim_matches('"');
    Ok(Session {
        bin_filename: conf.bin_filename.clone(),
        shorthand: conf.shorthand.clone(),
        session_code: SessionCode::V2KeyValuePair {
            key: key.to_string(),
            value: value.to_string(),
        },
        major_version: ReleaseNoteUrl::V2,
    })
}
