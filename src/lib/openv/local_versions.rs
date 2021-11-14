use std::path::Path;
use std::str::FromStr;

use thiserror::Error;
use tokio::fs;

use crate::openv::types::*;

#[derive(Debug, PartialEq, Error)]
enum LocalVersionError {
    #[error("can't find any local versions.")]
    NoLocalVersion,
}

async fn find_local_version(dirname: &Path) -> anyhow::Result<LocalVersion> {
    use LocalVersionError::*;
    let mut dir = fs::read_dir(dirname).await?;
    let mut xs = Vec::with_capacity(1024);
    let cp = Platform::current();
    while let Some(ent) = dir.next_entry().await? {
        let s = &ent.path().to_string_lossy().into_owned();
        if let Ok(lv) = LocalVersion::from_str(s) {
            if lv.platform == cp {
                xs.push(lv);
            }
        }
    }
    let opt_max = xs.into_iter().max_by(|l, r| l.version.cmp(&r.version));
    match opt_max {
        Some(mx) => Ok(mx),
        None => Err(anyhow::Error::new(NoLocalVersion)),
    }
}

#[cfg(test)]
mod test {
    use semver::Version;
    use tokio::runtime::Runtime;

    use super::*;

    #[test]
    fn test_find_local_version() {
        let p = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("fake_binaries");
        let fut = find_local_version(&p);
        let rt = Runtime::new().unwrap();
        let lv = rt.block_on(fut);
        assert!(lv.is_ok());
        let LocalVersion {
            version,
            platform,
            path,
        } = lv.unwrap();
        assert_eq!(Version::new(1, 11, 2), version);
        assert_eq!(
            Platform {
                os: OperatingSystem::Linux,
                arch: Arch::AMD64,
            },
            platform
        );
        assert!(path.ends_with("op_linux_amd64_v1.11.2"));
    }

    #[test]
    fn test_not_find_local_version() {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
        let fut = find_local_version(&p);
        let rt = Runtime::new().unwrap();
        let lv = rt.block_on(fut);
        assert!(lv.is_err());
    }
}
