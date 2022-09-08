use std::fs::Permissions;
use std::path::PathBuf;

use dirs::home_dir;
use tokio::fs;

#[cfg(target_family = "unix")]
async fn handle_permission(p: &PathBuf, perms: Permissions) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if !(perms.mode() == 0o100700 || perms.mode() == 0o700) {
        perms.set_mode(0o700);
        fs::set_permissions(&p, perms).await?;
    }
    Ok(())
}

#[cfg(target_family = "windows")]
async fn handle_permission(_p: &PathBuf, _perms: Permissions) -> std::io::Result<()> {
    Ok(())
}

pub async fn get_or_create() -> anyhow::Result<String> {
    const ONE_PASSWORD_HOME_DIR: &str = ".op_cli";
    let hd = home_dir()
        .ok_or_else(|| anyhow::anyhow!("failed to retrieve the home directory."))?
        .join(ONE_PASSWORD_HOME_DIR);
    let opt_metadata = fs::metadata(&hd).await;
    if opt_metadata.is_err() || !opt_metadata.unwrap().file_type().is_dir() {
        fs::create_dir(&hd).await?;
    }
    let metadata = fs::metadata(&hd).await?;
    #[allow(unused_mut)]
    let mut perms = metadata.permissions();
    handle_permission(&hd, perms).await?;
    Ok(hd.to_string_lossy().into_owned())
}

#[cfg(test)]
mod test {
    use tokio::runtime::Runtime;

    use super::*;

    #[test]
    fn test_get_home_dir() {
        let rt = Runtime::new().unwrap();
        let o = rt.block_on(get_or_create());
        assert!(o.is_ok());
    }
}
