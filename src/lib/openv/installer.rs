use std::path::Path;

use crate::openv::downloader::download_url;
use crate::openv::local_versions::find_local_version;
use crate::openv::op_release::{download_release_notes, parse_release_notes};
use crate::openv::types::*;
use crate::openv::unpacker::{unpack_apple_gzip, unpack_apple_pkg, unpack_one_to, UnpackOption};
use tokio::fs;

#[allow(dead_code)]
pub async fn get_or_install(dirname: &Path) -> anyhow::Result<Installation> {
    // in the future, it will compare the local version against the release version and install
    // the latest version.
    if let Ok(lv) = find_local_version(dirname).await {
        return Ok(Installation {
            local_version: lv,
            release: None,
        });
    }
    let rl_notes = download_release_notes().await?;
    let release = parse_release_notes(&rl_notes)?;
    let o_filename = download_url(dirname, &release.url).await?;
    let archive_filename = Path::new(&o_filename);

    let (_, binary_filename) = if cfg!(target_os = "macos") {
        let p = "/tmp/pkgutil_workdir";
        let _dont_care = fs::remove_dir_all(p);
        let gzip_filename = unpack_apple_pkg(archive_filename, p.as_ref())?;
        let basen = archive_filename
            .iter()
            .last()
            .ok_or(anyhow::anyhow!(
                "irregular path (no basename): {:?}",
                archive_filename
            ))?
            .to_str()
            .ok_or(anyhow::anyhow!(
                "irregular path (cannot convert to str): {:?}",
                archive_filename
            ))?;
        let (basen_clean, _) = basen
            .rsplit_once(".")
            .ok_or(anyhow::anyhow!("no file extension: {:?}", archive_filename))?;
        let o = unpack_apple_gzip(gzip_filename.as_ref(), &dirname, "op", Some(basen_clean))?;
        fs::remove_file(gzip_filename).await?;
        o
    } else {
        let unpack_opt = UnpackOption::UseArchiveName("op".to_string());
        unpack_one_to(&archive_filename, unpack_opt, &dirname)?
    };
    fs::remove_file(&archive_filename).await?;
    Ok(Installation {
        local_version: LocalVersion {
            version: release.version.clone(),
            platform: release.platform.clone(),
            path: binary_filename,
        },
        release: Some(release),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tokio::runtime::Runtime;

    #[test]
    fn test_ensure_installed_for_the_first_time() {
        // empty the directory
        let dirname: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "testdata",
            "tmp",
            "installed_for_the_first_time",
        ]
        .iter()
        .collect();
        let _dont_care = fs::remove_dir_all(&dirname);
        assert!(fs::create_dir_all(&dirname).is_ok());

        let fut = get_or_install(&dirname);
        let rt = Runtime::new().unwrap();
        let rs = rt.block_on(fut);
        println!("{:?}", rs);
        assert!(rs.is_ok());
        let inst = rs.unwrap();
        // has a release value
        assert!(inst.release.is_some());
        assert!(fs::remove_dir_all(&dirname).is_ok());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_ensure_get_preinstalled_binary() {
        // empty the directory
        let dirname: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "testdata",
            "tmp",
            "get_preinstalled_binary",
        ]
        .iter()
        .collect();
        assert!(fs::create_dir_all(&dirname).is_ok());

        let filename = dirname.clone().join("op_linux_amd64_v1.13.15");
        assert!(fs::File::create(&filename).is_ok());

        let fut = get_or_install(&dirname);
        let rt = Runtime::new().unwrap();
        let rs = rt.block_on(fut);
        assert!(rs.is_ok());
        let inst = rs.unwrap();
        // doesn't have the release value (local installation)
        assert!(inst.release.is_none());

        // assert_eq!(Version::new(1, 13, 15), inst.local_version.version);

        assert!(fs::remove_dir_all(&dirname).is_ok());
    }
}
