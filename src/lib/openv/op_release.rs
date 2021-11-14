use regex::Regex;
use semver::Version;
use std::array::IntoIter;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
enum HtmlParsingError {
    #[error("Missing html <body>...</body> tag.")]
    MissingBodyTag,

    #[error("Missing html <article>...</article> tag.")]
    MissingArticleTag,

    #[error("Missing the version string, e.g. title=\"1.12.3 - build #1120301\".")]
    MissingVersionString,

    #[error("Version string is not a semantic version (can not parse). A legit semver should look like 1.12.3.")]
    VersionStringIsNotSemver,

    #[error("Missing download urls to the binaries.")]
    MissingDownloadUrls,

    #[error("Download url does not contain target tokens. Expect: *_<OS>_<Arch>_*. Got: {0:?}.")]
    InvalidTargetUrl(String),
}

#[derive(Debug, PartialEq)]
struct Release {
    version: Version,
    targets: Targets,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum OperatingSystem {
    Apple,
    Linux,
    OpenBSD,
    FreeBSD,
    Windows,
}

impl FromStr for OperatingSystem {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use OperatingSystem::*;
        match s {
            "apple" => Ok(Apple),
            "linux" => Ok(Linux),
            "openbsd" => Ok(OpenBSD),
            "freebsd" => Ok(FreeBSD),
            "windows" => Ok(Windows),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Arch {
    X86_32,
    AMD64,
    Arm64,
    Arm,
    AppleUniversal,
}

impl FromStr for Arch {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Arch::*;
        match s {
            "386" => Ok(X86_32),
            "arm64" => Ok(Arm64),
            "amd64" => Ok(AMD64),
            "arm" => Ok(Arm),
            "universal" => Ok(AppleUniversal),
            _ => Err(()),
        }
    }
}

type Targets = BTreeMap<OperatingSystem, BTreeMap<Arch, String>>;

fn extract_latest_release(text: &str) -> anyhow::Result<&str> {
    use HtmlParsingError::*;
    let (_, html_body) = text.split_once("<body>").ok_or(MissingBodyTag)?;
    let (latest_release_info, _) = html_body
        .split_once("</article>")
        .ok_or(MissingArticleTag)?;
    Ok(latest_release_info)
}

fn extract_version(text: &str) -> anyhow::Result<Version> {
    use HtmlParsingError::*;
    let version_re = Regex::new(r"(\d+\.\d+\.\d+) - build #\d+").unwrap();
    let ver_str = version_re
        .captures(text)
        .map(|cap| cap.get(1).map(|mat| mat.as_str()))
        .flatten()
        .ok_or(MissingVersionString)?;
    Version::parse(ver_str).map_err(|_| anyhow::Error::new(VersionStringIsNotSemver))
}

fn extract_download_urls(text: &str) -> anyhow::Result<Vec<&str>> {
    use HtmlParsingError::*;
    let url_re = Regex::new(r####" href="(https.+?)" title="####).unwrap();
    let urls = url_re
        .captures_iter(text)
        .map(|cap| cap.get(1).map(|mat| mat.as_str()).unwrap())
        .collect::<Vec<_>>();
    if urls.is_empty() {
        Err(anyhow::Error::new(MissingDownloadUrls))
    } else {
        Ok(urls)
    }
}

fn parse_download_urls(urls: Vec<&str>) -> anyhow::Result<Targets> {
    use HtmlParsingError::*;
    let mut targets = Targets::new();
    let re = Regex::new(r####"op_([0-9a-zA-Z]+)_([0-9a-zA-Z]+)_v"####).unwrap();
    for url in urls {
        let (_, base) = url
            .rsplit_once("/")
            .ok_or(InvalidTargetUrl("no base".to_string()))?;
        let cap = re.captures(base).ok_or(InvalidTargetUrl(format!(
            "base has no os or arch: {}",
            base
        )))?;
        let os = OperatingSystem::from_str(cap.get(1).unwrap().as_str())
            .map_err(|_| InvalidTargetUrl(format!("invalid os: {}", base)))?;
        let arch = Arch::from_str(cap.get(2).unwrap().as_str())
            .map_err(|_| InvalidTargetUrl(format!("invalid arch: {}", base)))?;
        targets
            .entry(os)
            .and_modify(|ent| {
                ent.insert(arch, url.to_string());
            })
            .or_insert_with(|| {
                BTreeMap::<Arch, String>::from_iter(IntoIter::new([(arch, url.to_string())]))
            });
    }
    Ok(targets)
}

fn parse_release_notes(body: &str) -> anyhow::Result<Release> {
    let latest_release_info = extract_latest_release(body)?;
    let version = extract_version(latest_release_info)?;
    let download_urls = extract_download_urls(latest_release_info)?;
    let targets = parse_download_urls(download_urls)?;
    Ok(Release { version, targets })
}

#[allow(dead_code)]
const URL: &str = "https://app-updates.agilebits.com/product_history/CLI";

#[allow(dead_code)]
async fn download_release_notes() -> anyhow::Result<String> {
    let resp = reqwest::get(URL).await?;
    resp.text().await.map_err(|err| anyhow::Error::new(err))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::read_to_string;
    use std::path::Path;

    #[test]
    fn test_parse_release_notes_expect_successful() {
        let filename = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("release_notes")
            .join("2021_11_14_release_notes.html");
        let release_notes = read_to_string(filename).unwrap();
        let rl = parse_release_notes(&release_notes).unwrap();
        assert_eq!(semver::Version::new(1, 12, 3), rl.version);
        assert_eq!(5, rl.targets.len());
    }

    #[test]
    fn test_extract_latest_release_missing_body_tag() {
        let release_notes = r##"something fake"##;
        let result = extract_latest_release(release_notes);
        assert!(result.is_err());
        assert_eq!(
            &HtmlParsingError::MissingBodyTag,
            result
                .unwrap_err()
                .downcast_ref::<HtmlParsingError>()
                .unwrap()
        );
    }

    #[test]
    fn test_extract_latest_release_missing_article_tag() {
        let release_notes = r##"<body>
        </body>"##;
        let result = extract_latest_release(release_notes);
        assert!(result.is_err());
        assert_eq!(
            &HtmlParsingError::MissingArticleTag,
            result
                .unwrap_err()
                .downcast_ref::<HtmlParsingError>()
                .unwrap()
        );
    }

    #[test]
    fn test_extract_version_expect_error() {
        let release_notes = r##"something fake"##;
        let result = extract_version(release_notes);
        assert!(result.is_err());
        assert_eq!(
            &HtmlParsingError::MissingVersionString,
            result
                .unwrap_err()
                .downcast_ref::<HtmlParsingError>()
                .unwrap()
        );
    }

    #[test]
    fn test_extract_download_urls_expect_missing_urls_error() {
        let release_notes = r##"something fake"##;
        let result = extract_download_urls(release_notes);
        assert!(result.is_err());
        assert_eq!(
            &HtmlParsingError::MissingDownloadUrls,
            result
                .unwrap_err()
                .downcast_ref::<HtmlParsingError>()
                .unwrap()
        );
    }

    #[test]
    fn test_parse_download_urls_no_base() {
        let urls = vec!["http"];
        let result = parse_download_urls(urls);
        assert!(result.is_err());
        assert_eq!(
            &HtmlParsingError::InvalidTargetUrl("no base".to_string()),
            result
                .unwrap_err()
                .downcast_ref::<HtmlParsingError>()
                .unwrap()
        );
    }

    #[test]
    fn test_parse_download_urls_missing_os_arch() {
        let urls = vec!["https://some/v123.zip"];
        let result = parse_download_urls(urls);
        assert!(result.is_err());
        assert_eq!(
            &HtmlParsingError::InvalidTargetUrl("base has no os or arch: v123.zip".to_string()),
            result
                .unwrap_err()
                .downcast_ref::<HtmlParsingError>()
                .unwrap()
        );
    }

    #[test]
    fn test_parse_download_urls_invalid_os() {
        let urls = vec!["https://some/op_snes_16bit_v122.zip"];
        let result = parse_download_urls(urls);
        assert!(result.is_err());
        assert_eq!(
            &HtmlParsingError::InvalidTargetUrl("invalid os: op_snes_16bit_v122.zip".to_string()),
            result
                .unwrap_err()
                .downcast_ref::<HtmlParsingError>()
                .unwrap()
        );
    }

    #[test]
    fn test_parse_download_urls_invalid_arch() {
        let urls = vec!["https://some/op_linux_16bit_v122.zip"];
        let result = parse_download_urls(urls);
        assert!(result.is_err());
        assert_eq!(
            &HtmlParsingError::InvalidTargetUrl(
                "invalid arch: op_linux_16bit_v122.zip".to_string()
            ),
            result
                .unwrap_err()
                .downcast_ref::<HtmlParsingError>()
                .unwrap()
        );
    }
}
