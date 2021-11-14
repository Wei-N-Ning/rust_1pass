use anyhow::anyhow;
use regex::Regex;
use std::env;
use std::fmt::Debug;
use std::str::FromStr;

use semver::Version;
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum HtmlParsingError {
    #[error("Missing html <body>...</body> tag.")]
    MissingBodyTag,

    #[error("Missing html <article>...</article> tag.")]
    MissingArticleTag,

    #[error("Missing download urls to the binaries.")]
    MissingDownloadUrls,

    #[error("Missing platform. Expect: {0:?}")]
    MissingPlatform(Platform),
}

#[derive(Debug, PartialEq)]
pub struct Release {
    pub version: Version,
    pub platform: Platform,
    pub url: String,
}

impl FromStr for Release {
    type Err = anyhow::Error;

    /// Expect: https://.../op_<os>_<arch>_v<version>*
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let base = match s.rsplit_once("/") {
            Some((_, x)) => x,
            None => s,
        };
        let re = Regex::new("^op_([0-9a-zA-Z]+_[0-9a-zA-Z]+)_v([0-9.]+[0-9])").unwrap();
        let captures = re
            .captures(base)
            .ok_or(anyhow!("invalid format: {}", base))?;
        let platform = Platform::from_str(captures.get(1).unwrap().as_str())?;
        let version = Version::from_str(captures.get(2).unwrap().as_str())?;
        Ok(Release {
            version,
            platform,
            url: s.to_string(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct LocalVersion {
    pub version: Version,
    pub platform: Platform,
    pub path: String,
}

impl FromStr for LocalVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let basename = match s.rsplit_once(|ch| ch == '/' || ch == '\\') {
            Some((_, x)) => x,
            None => s,
        };
        let rl = Release::from_str(basename)?;
        Ok(LocalVersion {
            version: rl.version,
            platform: rl.platform,
            path: s.to_string(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperatingSystem {
    Apple,
    Linux,
    OpenBSD,
    FreeBSD,
    Windows,
}

impl OperatingSystem {
    fn current() -> Self {
        Self::from_str(env::consts::OS).unwrap()
    }
}

impl FromStr for OperatingSystem {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use OperatingSystem::*;
        match s {
            "apple" => Ok(Apple),
            "linux" => Ok(Linux),
            "openbsd" => Ok(OpenBSD),
            "freebsd" => Ok(FreeBSD),
            "windows" => Ok(Windows),
            _ => Err(anyhow!("unsupported os: {}", s)),
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

impl Arch {
    #[allow(unreachable_code)]
    fn current() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            return Arch::AMD64;
        }
        #[cfg(target_arch = "x86")]
        {
            return Arch::X86_32;
        }
        #[cfg(target_arch = "arm")]
        {
            return Arch::Arm;
        }
        unimplemented!("")
    }
}

impl FromStr for Arch {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Arch::*;
        match s {
            "386" => Ok(X86_32),
            "arm64" => Ok(Arm64),
            "amd64" => Ok(AMD64),
            "arm" => Ok(Arm),
            "universal" => Ok(AppleUniversal),
            _ => Err(anyhow!("unsupported arch: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Platform {
    pub os: OperatingSystem,
    pub arch: Arch,
}

impl Platform {
    pub(crate) fn current() -> Self {
        Self {
            os: OperatingSystem::current(),
            arch: Arch::current(),
        }
    }
}

impl FromStr for Platform {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (os_str, arch_str) = s
            .split_once("_")
            .ok_or(anyhow!("missing delimiter: {}", s))?;
        let os = OperatingSystem::from_str(os_str)?;
        let arch = match os {
            OperatingSystem::Apple => {
                if arch_str == "universal" {
                    Arch::AppleUniversal
                } else {
                    return Err(anyhow!(
                        "only support 'universal' arch for apple, got: {}",
                        arch_str
                    ));
                }
            }
            _ => Arch::from_str(arch_str)?,
        };
        Ok(Self { os, arch })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ensure_current_operating_system_recognizable() {
        let os = OperatingSystem::current();
        assert!(format!("{:?}", os).len() > 0);
    }

    #[test]
    fn test_ensure_current_arch_recognizable() {
        let _ = Arch::current();
    }

    #[test]
    fn test_ensure_current_platform_recognizable() {
        let _ = Platform::current();
    }

    #[test]
    fn test_parse_os_string() {
        assert!(OperatingSystem::from_str("snes").is_err());
        assert!(OperatingSystem::from_str("apple").is_ok());
    }

    #[test]
    fn test_parse_architecture_string() {
        assert!(Arch::from_str("486").is_err());
        assert!(Arch::from_str("386").is_ok());
    }

    #[test]
    fn test_parse_platform_string() {
        assert!(Platform::from_str("apple_universal").is_ok());
        assert!(Platform::from_str("apple_x86").is_err());

        assert!(Platform::from_str("linux_amd64").is_ok());
    }

    #[test]
    fn test_parse_release_expect_successful() {
        let u = "https://cache.agilebits.com/dist/1P/op/pkg/v1.12.1/op_freebsd_386_v1.12.1.zip";
        let rl = Release::from_str(u).unwrap();
        assert_eq!(
            Platform {
                os: OperatingSystem::FreeBSD,
                arch: Arch::X86_32
            },
            rl.platform
        );
        assert_eq!(Version::new(1, 12, 1), rl.version);
        assert_eq!(u, &rl.url);
    }

    #[test]
    fn test_parse_release_expect_error() {
        let u = "https://cache.agilebits.com/dist/1P/op/pkg/v1.12.1/op_freebsd_586_v1.12.1.zip";
        let rl = Release::from_str(u);
        assert!(rl.is_err());
    }

    #[test]
    fn test_parse_local_version_windows_path() {
        let p = r"C:\我的\文档\op_freebsd_386_v1.12.1";
        let lv = LocalVersion::from_str(p);
        assert!(lv.is_ok());
        let LocalVersion {
            version,
            platform,
            path,
        } = lv.unwrap();
        assert_eq!(Version::new(1, 12, 1), version);
        assert_eq!(
            Platform {
                os: OperatingSystem::FreeBSD,
                arch: Arch::X86_32
            },
            platform
        );
        assert_eq!(p, &path);
    }

    #[test]
    fn test_parse_posix_local_version_path() {
        let p = "/usr/home/files/op_freebsd_386_v1.12.1";
        let lv = LocalVersion::from_str(p);
        assert!(lv.is_ok());
        let LocalVersion {
            version,
            platform,
            path,
        } = lv.unwrap();
        assert_eq!(Version::new(1, 12, 1), version);
        assert_eq!(
            Platform {
                os: OperatingSystem::FreeBSD,
                arch: Arch::X86_32
            },
            platform
        );
        assert_eq!(p, &path);
    }
}
