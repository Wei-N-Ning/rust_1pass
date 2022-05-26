use std::process::Command;
use std::str::FromStr;

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

#[derive(Debug)]
pub struct Account {
    pub shorthand: String, // e.g. iddqd
    pub email: String,     // e.g. doomguy@doom.org
    pub op_url: String,    // e.g. https://my.1password.com
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
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<_>>())
    }
}

impl Account {
    pub fn from_descriptions(desc: &str) -> Vec<Account> {
        let mut xs = Vec::new();
        for line in desc.lines() {
            if let Ok(acc) = Account::from_str(line) {
                xs.push(acc)
            }
        }
        xs
    }
}

impl FromStr for Account {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let xs = s.split_whitespace().collect::<Vec<_>>();
        if let [_, shorthand, email, op_url] = xs[..] {
            if email.contains('@') && op_url.starts_with("https://") {
                return Ok(Account {
                    shorthand: shorthand.to_owned(),
                    email: email.to_owned(),
                    op_url: op_url.to_owned(),
                });
            }
        }
        Err(anyhow::anyhow!("cannot parse account description: {}", s))
    }
}

#[cfg(test)]
mod test {
    use crate::session::types::Account;
    use std::str::FromStr;

    #[test]
    fn test_parse_empty_line_expect_error() {
        assert!(Account::from_str("").is_err());
        assert!(Account::from_str("\t\n").is_err());
        assert!(Account::from_str("   \t   \n").is_err());
        assert!(Account::from_str("  \t \t \t \t   \n").is_err());
    }

    #[test]
    fn test_parse_account_description_expect_success() {
        let desc = "      1. my	doomguy@doomorg.com	 https://my.1password.com";
        let result = Account::from_str(desc);
        assert!(result.is_ok());
        let acc = result.unwrap();
        assert_eq!(acc.op_url, "https://my.1password.com");
        assert_eq!(acc.email, "doomguy@doomorg.com");
        assert_eq!(acc.shorthand, "my");
    }

    #[test]
    fn test_parse_account_description_expect_newline_stripped() {
        let desc = "      1. my	doomguy@doomorg.com	 https://my.1password.com\n\n\n\n";
        let result = Account::from_str(desc);
        assert!(result.is_ok());
        let acc = result.unwrap();
        assert_eq!(acc.op_url, "https://my.1password.com");
        assert_eq!(acc.email, "doomguy@doomorg.com");
        assert_eq!(acc.shorthand, "my");
    }

    #[test]
    fn test_parse_account_description_expect_error() {
        let desc = "Accounts on this device:";
        let result = Account::from_str(desc);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_descriptions_expect_num_accounts() {
        let desc = r###"
Accounts on this device:
   
      1. my	macgnw@gmail.com	https://my.1password.com
      2. home	macgnw@gmail.com	https://my.1password.com
      3. immu	wei.ning@immutable.com	https://my.1password.com
        "###;

        let accounts = Account::from_descriptions(desc);
        assert_eq!(3, accounts.len());
    }
}
