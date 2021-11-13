use crate::openv::op_release::HtmlParsingError::InvalidTargetUrl;
use regex::Regex;
use semver::Version;
use std::array::IntoIter;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use tokio::runtime;

const URL: &str = "https://app-updates.agilebits.com/product_history/CLI";

async fn get_some() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get(URL).await?;
    println!("{:?}", resp.content_length());
    let body = resp.text().await?;
    let parsed = parse(&body);
    println!("{:?}", parsed);
    Ok(())
}

#[derive(Debug, PartialEq)]
enum HtmlParsingError {
    MissingBodyTag,
    MissingArticleTag,
    MissingVersionString,
    VersionStringIsNotSemver,
    MissingDownloadUrls,
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

impl Display for HtmlParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for HtmlParsingError {}

fn extract_latest_release(text: &str) -> Result<&str, HtmlParsingError> {
    use HtmlParsingError::*;
    let (_, html_body) = text.split_once("<body>").ok_or(MissingBodyTag)?;
    let (latest_release_info, _) = html_body
        .split_once("</article>")
        .ok_or(MissingArticleTag)?;
    Ok(latest_release_info)
}

fn extract_version(text: &str) -> Result<Version, HtmlParsingError> {
    use HtmlParsingError::*;
    let version_re = Regex::new(r"(\d+\.\d+\.\d+) - build #\d+").unwrap();
    let ver_str = version_re
        .captures(text)
        .map(|cap| cap.get(1).map(|mat| mat.as_str()))
        .flatten()
        .ok_or(MissingVersionString)?;
    Version::parse(ver_str).map_err(|_| VersionStringIsNotSemver)
}

fn extract_download_urls(text: &str) -> Result<Vec<&str>, HtmlParsingError> {
    use HtmlParsingError::*;
    let url_re = Regex::new(r####" href="(https.+?)" title="####).unwrap();
    let urls = url_re
        .captures_iter(text)
        .map(|cap| cap.get(1).map(|mat| mat.as_str()).unwrap())
        .collect::<Vec<_>>();
    if urls.is_empty() {
        Err(MissingDownloadUrls)
    } else {
        Ok(urls)
    }
}

fn parse_download_urls(urls: Vec<&str>) -> Result<Targets, HtmlParsingError> {
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

fn parse(body: &str) -> Result<Release, HtmlParsingError> {
    let latest_release_info = extract_latest_release(body)?;
    let version = extract_version(latest_release_info)?;
    let download_urls = extract_download_urls(latest_release_info)?;
    let targets = parse_download_urls(download_urls)?;
    Ok(Release { version, targets })
}

#[test]
fn test_get_some() {
    let fut = get_some();
    let rt = runtime::Runtime::new().unwrap();
    let o = rt.block_on(fut);
    assert!(o.is_ok());
}
