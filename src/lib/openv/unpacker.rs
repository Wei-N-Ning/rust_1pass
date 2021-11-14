use anyhow::anyhow;
use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

enum UnpackOption {
    /// name the unpacked file after the zip archive entry
    UseEntryName(String),
    /// name the unpacked file after the zip archive (without the extension)
    UseArchiveName(String),
}

fn unpack_one_to(zfilename: &Path, opt: UnpackOption, o_dir: &Path) -> anyhow::Result<String> {
    let zipfile = std::fs::File::open(&zfilename).unwrap();
    let mut archive = zip::ZipArchive::new(zipfile)?;
    let (o_filename, file) = match opt {
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
    let mut contents = Vec::<u8>::with_capacity(1024);
    BufReader::new(file).read(&mut contents)?;

    let o_file = fs::File::create(&o_filename)?;
    BufWriter::new(o_file).write_all(&contents)?;
    return Ok(o_filename.to_string_lossy().into_owned());
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
        assert!(res.unwrap().ends_with("op"));
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
        assert!(res.unwrap().ends_with("op_linux_amd64_v1.11.2"));
    }
}
