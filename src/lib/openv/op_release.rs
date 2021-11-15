// to download and parse the 1password CLI release note;
// to extract the latest release version and compose a btree-map of:
// { os: { arch: download_url } }
// e.g.
// { apple: { universal: https://..../op_apple_universal_v1.12.3.zip }
//   linux: { 386: ..., AMD64: ... }
//   ...
// }

use crate::openv::types::*;
use regex::Regex;
use std::str::FromStr;

fn extract_latest_release(text: &str) -> anyhow::Result<&str> {
    use HtmlParsingError::*;
    let (_, html_body) = text.split_once("<body>").ok_or(MissingBodyTag)?;
    let (latest_release_info, _) = html_body
        .split_once("</article>")
        .ok_or(MissingArticleTag)?;
    Ok(latest_release_info)
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

fn parse_download_urls(urls: Vec<&str>) -> anyhow::Result<Release> {
    use HtmlParsingError::*;
    let cp = Platform::current();
    for url in urls {
        if let Ok(rl) = Release::from_str(url) {
            if rl.platform == cp {
                return Ok(rl);
            }
        }
    }
    Err(anyhow::Error::new(MissingPlatform(cp)))
}

pub fn parse_release_notes(body: &str) -> anyhow::Result<Release> {
    let latest_release_info = extract_latest_release(body)?;
    let download_urls = extract_download_urls(latest_release_info)?;
    parse_download_urls(download_urls)
}

const URL: &str = "https://app-updates.agilebits.com/product_history/CLI";

pub async fn download_release_notes() -> anyhow::Result<String> {
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
            &HtmlParsingError::MissingPlatform(Platform {
                os: OperatingSystem::Linux,
                arch: Arch::AMD64
            }),
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
            &HtmlParsingError::MissingPlatform(Platform {
                os: OperatingSystem::Linux,
                arch: Arch::AMD64
            }),
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
            &HtmlParsingError::MissingPlatform(Platform {
                os: OperatingSystem::Linux,
                arch: Arch::AMD64
            }),
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
            &HtmlParsingError::MissingPlatform(Platform {
                os: OperatingSystem::Linux,
                arch: Arch::AMD64
            }),
            result
                .unwrap_err()
                .downcast_ref::<HtmlParsingError>()
                .unwrap()
        );
    }
}
