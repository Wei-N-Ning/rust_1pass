use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{fs, io};

use anyhow::anyhow;

pub enum UnpackOption {
    /// name the unpacked file after the zip archive entry
    UseEntryName(String),
    /// name the unpacked file after the zip archive (without the extension)
    UseArchiveName(String),
}

pub fn unpack_one_to(
    zfilename: &Path,
    opt: UnpackOption,
    o_dir: &Path,
) -> anyhow::Result<(u64, String)> {
    let zipfile = std::fs::File::open(&zfilename).unwrap();
    let mut archive = zip::ZipArchive::new(zipfile)?;
    let (o_filename, mut file) = match opt {
        UnpackOption::UseEntryName(name) => (o_dir.join(&name), archive.by_name(&name)?),
        UnpackOption::UseArchiveName(name) => {
            let basename = zfilename
                .file_stem()
                .ok_or(anyhow!("irregular filename: {:?}", zfilename))?
                .to_string_lossy()
                .into_owned();
            (o_dir.join(basename), archive.by_name(&name)?)
        }
    };
    let mut o_file = fs::File::create(&o_filename)?;
    let copied = io::copy(&mut file, &mut o_file)?;
    let mut perms = fs::metadata(&o_filename)?.permissions();
    perms.set_mode(0o700);
    fs::set_permissions(&o_filename, perms)?;
    return Ok((copied, o_filename.to_string_lossy().into_owned()));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_unpack_single_file_to_dest_use_entry_name() {
        let zfilename = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("archives")
            .join("op_linux_amd64_v1.11.2.zip");
        let tmp = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("tmp");
        let res = unpack_one_to(
            &zfilename,
            UnpackOption::UseEntryName("op".to_string()),
            &tmp,
        );
        assert!(res.is_ok());
        assert!(res.unwrap().1.ends_with("op"));
    }

    #[test]
    fn test_unpack_single_file_to_dest_use_zipfile_name() {
        let zfilename = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("archives")
            .join("op_linux_amd64_v1.11.2.zip");
        let tmp = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("tmp");
        let res = unpack_one_to(
            &zfilename,
            UnpackOption::UseArchiveName("op".to_string()),
            &tmp,
        );
        assert!(res.is_ok());
        assert!(res.unwrap().1.ends_with("op_linux_amd64_v1.11.2"));
    }

    #[test]
    fn test_unpack_apple_pkg() {
        let pkg_filename: std::path::PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "testdata",
            "archives",
            "op_apple_universal_v1.12.4.pkg",
        ]
        .iter()
        .collect();
        let mut input = io::BufReader::new(Box::new(fs::File::open(pkg_filename).unwrap()));
        let mut decoder =
            libflate::gzip::Decoder::new(input).expect("failed to read .pkg (gzip) file!");
    }
}
