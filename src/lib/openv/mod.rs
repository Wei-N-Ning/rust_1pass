#[allow(dead_code)]
mod downloader;

#[allow(dead_code)]
mod local_versions;

#[allow(dead_code)]
mod op_release;

#[allow(dead_code)]
mod types;

mod installer;
#[allow(dead_code)]
mod unpacker;

pub use installer::get_or_install;
pub use types::Installation;
