use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing_test::traced_test;

const ELECTRON_RELEASES: &str = "https://releases.electronjs.org/releases.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct ElectronVersion {
    /// Electron version
    pub version: semver::Version,

    /// Date of release
    pub date: NaiveDate,

    /// Version of Node.js
    pub node: String,

    /// Version of V8
    pub v8: String,

    /// Version of uv
    pub uv: String,

    /// OpenSSL version
    pub openssl: String,

    /// ABI version
    #[serde(rename = "modules")]
    pub abi_version: String,

    /// Release files
    pub files: Vec<ReleaseType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReleaseType {
    // darwin-x64
    #[serde(rename = "darwin-x64")]
    DarwinX64,
    #[serde(rename = "darwin-x64-symbols")]
    DarwinX64Symbols,
    #[serde(rename = "linux-ia32")]
    LinuxIa32,
    #[serde(rename = "linux-ia32-symbols")]
    LinuxIa32Symbols,
    #[serde(rename = "linux-x64")]
    LinuxX64,
    #[serde(rename = "linux-x64-symbols")]
    LinuxX64Symbols,
    #[serde(rename = "win32-ia32")]
    Win32Ia32,
    #[serde(rename = "win32-ia32-symbols")]
    Win32Ia32Symbols,
    #[serde(rename = "win32-x64")]
    Win32X64,
    #[serde(rename = "win32-x64-symbols")]
    Win32X64Symbols,

    // names other than this will be Other
    #[serde(other)]
    Other,
}

pub fn cache() -> Result<cached_path::Cache> {
    Ok(cached_path::Cache::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()?)
}

pub fn cached_file() -> Result<PathBuf> {
    let releases_file = cache()?.cached_path(ELECTRON_RELEASES)?;
    Ok(releases_file)
}

/// Literally just Vec<ElectronVersion>
#[derive(Debug, Serialize, Deserialize)]
pub struct ElectronReleases {
    #[serde(flatten)]
    pub releases: Vec<ElectronVersion>,
}

impl ElectronReleases {
    pub fn load() -> Result<Self> {
        let releases_file = cached_file()?;
        let releases = std::fs::read_to_string(releases_file)?;
        // tracing::debug!("releases: {}", releases);
        let releases: Vec<ElectronVersion> = serde_json::from_str(&releases)?;
        Ok(Self { releases })
    }
}

#[traced_test]
#[test]
fn test_electron_releases() -> Result<()> {
    let releases = ElectronReleases::load()?;
    tracing::info!("releases: {:#?}", releases);
    Ok(())
}
