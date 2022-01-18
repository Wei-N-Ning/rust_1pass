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
    let zipfile = std::fs::File::open(&zfilename)?;
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

pub fn unpack_apple_pkg(pkg_filename: &Path, o_dir: &Path) -> anyhow::Result<String> {
    let mut proc = std::process::Command::new("pkgutil")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .arg("--expand")
        .arg(&pkg_filename)
        .arg(&o_dir)
        .spawn()?;
    proc.wait()?;

    let payload = o_dir.join("op.pkg").join("Payload");
    if fs::metadata(&payload)?.is_file() {
        Ok(payload.to_string_lossy().into_owned())
    } else {
        Err(anyhow!(
            "failed to extract op.pkg/Payload from apple installer file: {:?}",
            &pkg_filename
        ))
    }
}

pub fn unpack_apple_gzip(
    gz_filename: &Path,
    o_dir: &Path,
    o_name: &str,
    rename: Option<&str>,
) -> anyhow::Result<(u64, String)> {
    let input = io::BufReader::new(fs::File::open(gz_filename)?);
    let mut decoder =
        libflate::gzip::Decoder::new(input).expect("failed to read gzip (.pkg Payload) file!");
    let cpio_filename = o_dir.clone().join("out.cpio");
    let mut output = io::BufWriter::new(fs::File::create(cpio_filename)?);
    io::copy(&mut decoder, &mut output)?;
    let mut proc = std::process::Command::new("cpio")
        .current_dir(&o_dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .arg("-i")
        .arg("-F")
        .arg("/tmp/out.cpio")
        .spawn()?;
    proc.wait()?;
    let o_filename = o_dir.join(o_name);
    if std::fs::metadata(&o_filename)?.is_file() {
        if let Some(x) = rename {
            let dest_filename = o_dir.join(x);
            fs::rename(&o_filename, &dest_filename)?;
            Ok((0, dest_filename.to_string_lossy().into_owned()))
        } else {
            Ok((0, o_filename.to_string_lossy().into_owned()))
        }
    } else {
        Err(anyhow!(
            "failed to extract '{}' from cpio archive '{:?}'",
            o_name,
            &o_filename
        ))
    }
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
    fn test_unpack_apple_gzip_file() {
        let gzip_filename: std::path::PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "testdata",
            "archives",
            "apple_pkg_expanded_gzip",
        ]
        .iter()
        .collect();
        let (_, o_filename) = unpack_apple_gzip(
            gzip_filename.as_ref(),
            "/tmp".as_ref(),
            "op",
            Some("op_there_is_a_cow_1.1.2"),
        )
        .unwrap();

        assert!(std::fs::metadata(o_filename).unwrap().is_file());
    }
}
