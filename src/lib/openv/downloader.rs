use anyhow::anyhow;
use reqwest::StatusCode;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[allow(dead_code)]
pub async fn download_url(o_dir: &Path, u: &str) -> anyhow::Result<String> {
    let res = reqwest::get(u).await?;
    if res.status() != StatusCode::OK {
        return Err(anyhow!("request has been rejected: {}", u));
    }
    let basename = match u.rsplit_once("/") {
        Some((_, x)) => x,
        None => "unnamed",
    };
    let o_filename = o_dir.join(basename);
    let mut o_file = fs::File::create(&o_filename).await?;
    let body = res.bytes().await?;
    o_file.write_all(&body).await?;
    Ok(o_filename.to_string_lossy().into_owned())
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_download_url() {
        let u = "https://cache.agilebits.com/dist/1P/op/pkg/v1.12.1/op_linux_amd64_v1.12.1.zip";
        let rt = Runtime::new().unwrap();
        let o_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("tmp");
        let fut = download_url(&o_dir, u);
        let result = rt.block_on(fut);
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("op_linux_amd64_v1.12.1.zip"));
    }
}
