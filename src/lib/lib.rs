mod openv;
mod session;

use openv::*;
use session::*;

// prelude

pub async fn list_local_accounts() -> anyhow::Result<()> {
    let home_dir = get_or_create().await?;
    let inst = get_or_install(std::path::Path::new(&home_dir), ReleaseNoteUrl::V2).await?;
    let sess_conf = SessionConfig {
        bin_filename: inst.local_version.path,
        shorthand: String::new(),
    };
    let accounts = match inst.major_version {
        ReleaseNoteUrl::V1 => local_accounts_v1(&sess_conf)?,
        ReleaseNoteUrl::V2 => local_accounts_v2(&sess_conf)?,
    };
    for ref acc in accounts {
        println!("{:?}", acc);
    }
    Ok(())
}

pub async fn make_session(shorthand: &str) -> anyhow::Result<Session> {
    let home_dir = get_or_create().await?;
    let inst = get_or_install(std::path::Path::new(&home_dir), ReleaseNoteUrl::V2).await?;
    let sess_conf = SessionConfig {
        bin_filename: inst.local_version.path,
        shorthand: shorthand.to_string(),
    };
    match inst.major_version {
        ReleaseNoteUrl::V1 => sign_in_shorthand_v1(&sess_conf),
        ReleaseNoteUrl::V2 => sign_in_shorthand_v2(&sess_conf),
    }
}
