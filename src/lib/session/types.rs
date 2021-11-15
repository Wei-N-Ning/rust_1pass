use std::process::Command;

pub struct SessionConfig {
    pub bin_filename: String,
    pub shorthand: String,
}

#[derive(Debug, PartialEq)]
pub struct Session {
    pub bin_filename: String,
    pub shorthand: String,
    pub session_code: String,
}

impl Session {
    pub fn item_fields(&self, item: &str, fields: &[&str]) -> anyhow::Result<Vec<String>> {
        let out = Command::new(&self.bin_filename)
            .env(format!("OP_SESSION_{}", self.shorthand), &self.session_code)
            .arg("get")
            .arg("item")
            .arg(item)
            .arg(format!("--fields={}", fields.join(",")))
            .arg("--format=CSV")
            .output()?;
        let s = String::from_utf8(out.stdout)?;
        Ok(s.trim()
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<_>>())
    }
}
