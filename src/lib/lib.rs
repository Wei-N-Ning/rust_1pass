mod openv;
mod session;

use openv::*;
use session::*;

// prelude

pub async fn make_session(shorthand: &str) -> anyhow::Result<Session> {
    let home_dir = get_or_create().await?;
    let inst = get_or_install(&std::path::Path::new(&home_dir)).await?;
    let sess_conf = SessionConfig {
        bin_filename: inst.local_version.path,
        shorthand: shorthand.to_string(),
    };
    sign_in_shorthand(&sess_conf)
}
