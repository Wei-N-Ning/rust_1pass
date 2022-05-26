#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum ReleaseNoteUrl {
    V1,
    V2,
}

impl ToString for ReleaseNoteUrl {
    fn to_string(&self) -> String {
        match self {
            ReleaseNoteUrl::V1 => {
                "https://app-updates.agilebits.com/product_history/CLI".to_owned()
            }
            ReleaseNoteUrl::V2 => {
                "https://app-updates.agilebits.com/product_history/CLI2".to_owned()
            }
        }
    }
}
