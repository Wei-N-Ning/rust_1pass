mod openv;
mod session;

use openv::*;
use session::*;

// prelude

pub async fn list_local_accounts() -> anyhow::Result<()> {
    let home_dir = get_or_create().await?;
    let inst = get_or_install(&std::path::Path::new(&home_dir), ReleaseNoteUrl::V2).await?;
    let sess_conf = SessionConfig {
        bin_filename: inst.local_version.path,
        shorthand: String::new(),
    };
    let accounts = local_accounts(&sess_conf)?;
    for ref acc in accounts {
        println!("{:?}", acc);
    }
    Ok(())
}

pub async fn make_session(shorthand: &str) -> anyhow::Result<Session> {
    let home_dir = get_or_create().await?;
    let inst = get_or_install(&std::path::Path::new(&home_dir), ReleaseNoteUrl::V2).await?;
    let sess_conf = SessionConfig {
        bin_filename: inst.local_version.path,
        shorthand: shorthand.to_string(),
    };
    sign_in_shorthand(&sess_conf)
}
