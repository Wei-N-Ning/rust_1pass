/// Configuration for the 1Password cli installer
pub struct OnePasswordInstallerConfig {}

/// Configuration for the 1Password session
pub struct OnePasswordConfig {
    pub binary_path: String,
    pub inst_config: Option<OnePasswordInstallerConfig>,
}

impl OnePasswordConfig {
    pub fn new(binary_path: String) -> Self {
        OnePasswordConfig {
            binary_path,
            inst_config: None,
        }
    }
}
