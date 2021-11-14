use crate::openv::types::*;
use regex::Regex;
use semver::Version;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error, PartialEq)]
enum LocalVersionError {
    #[error("Failed to get the directory's contents: {0:?}.")]
    InvalidDirectory(String),
}

fn basename(path: &Path) -> &OsStr {
    match path.iter().last() {
        Some(p) => p,
        None => path.as_os_str(),
    }
}

// fn extract_version(path: &Path) -> Option<Version> {
//     let re = Regex::new("op_").unwrap();
// }

#[allow(dead_code)]
async fn find_local_versions(dirname: &Path) -> anyhow::Result<Vec<String>> {
    use LocalVersionError::*;
    let mut xs = Vec::<String>::with_capacity(1024);
    let read_dir = fs::read_dir(dirname)?;
    for result_ent in read_dir {
        match result_ent {
            Ok(ent) => xs.push(ent.path().to_string_lossy().into_owned()),
            _ => (),
        }
    }
    Ok(xs)
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_get_basename() {
        assert_eq!(
            "file".to_string(),
            basename(&Path::new("/this/is/a/file"))
                .to_string_lossy()
                .into_owned()
        );

        assert_eq!(
            "thereisacow".to_string(),
            basename(&Path::new("thereisacow"))
                .to_string_lossy()
                .into_owned()
        );

        assert_eq!(
            "".to_string(),
            basename(&Path::new("")).to_string_lossy().into_owned()
        );
    }

    #[test]
    fn test_find_all_local_versions() {
        let rt = Runtime::new().unwrap();
        let dirname = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("fake_binaries");
        let xs = rt.block_on(find_local_versions(&dirname)).unwrap();
        println!("{:?}", xs);
    }
}
