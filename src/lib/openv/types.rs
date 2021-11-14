use semver::Version;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum HtmlParsingError {
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
pub struct Release {
    pub version: Version,
    pub targets: Targets,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperatingSystem {
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
pub enum Arch {
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

pub type Targets = BTreeMap<OperatingSystem, BTreeMap<Arch, String>>;
