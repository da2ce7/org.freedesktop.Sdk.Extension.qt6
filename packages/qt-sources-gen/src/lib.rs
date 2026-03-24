use std::io::Read;

use serde::Serialize;
use sha2::{Digest, Sha512};
use tracing::{debug, error, info, warn};

pub const BASE_URL: &str = "https://download.qt.io/official_releases/qt";

pub const MODULES: &[&str] = &[
    "qtbase",
    "qtshadertools",
    "qtdeclarative",
    "qttools",
    "qtsvg",
    "qtimageformats",
    "qtwayland",
    "qtmultimedia",
    "qt5compat",
    "qtwebsockets",
    "qtwebchannel",
    "qtpositioning",
    "qtlocation",
    "qtconnectivity",
    "qtserialport",
    "qtsensors",
    "qtserialbus",
    "qtcharts",
    "qtnetworkauth",
    "qthttpserver",
    "qtlanguageserver",
    "qtscxml",
    "qtremoteobjects",
    "qtquicktimeline",
    "qtquick3d",
    "qtquick3dphysics",
    "qtspeech",
    "qtvirtualkeyboard",
    "qttranslations",
    "qtgrpc",
    "qtlottie",
    "qt3d",
    "qtdatavis3d",
    "qtgraphs",
    "qtopenapi",
    "qtcanvaspainter",
    "qttasktree",
];

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CheckResult {
    /// Sources file matches upstream — nothing to do
    UpToDate,
    /// Sentinel changed — new sources content is provided
    Changed { new_sources: String },
    /// Something went wrong
    Error(String),
}

/// Check whether the cached `sources` file is still current by comparing the
/// SHA-512 of Qt's upstream `md5sums.txt` against the sentinel stored in the file.
///
/// If `dry_run` is true, only the sentinel is checked — archive hashes are not recomputed.
pub fn check_sources(version: &str, sources_path: &str, dry_run: bool) -> CheckResult {
    let Some((major_minor, _)) = version.rsplit_once('.') else {
        return CheckResult::Error("version must be in X.Y.Z format".into());
    };

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_mins(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CheckResult::Error(format!("failed to build HTTP client: {e}")),
    };

    // Fetch and hash md5sums.txt
    let md5sums_url = format!("{BASE_URL}/{major_minor}/{version}/submodules/md5sums.txt");
    info!(url = %md5sums_url, "fetching md5sums.txt");

    let md5sums_hash = match download_and_hash(&client, &md5sums_url) {
        Ok(h) => h,
        Err(e) => return CheckResult::Error(format!("failed to fetch md5sums.txt: {e}")),
    };

    let current_sentinel = format!("SHA512 (md5sums.txt) = {md5sums_hash}");
    info!(sentinel = %current_sentinel, "computed sentinel");

    // Compare with stored sentinel
    let existing = std::fs::read_to_string(sources_path).unwrap_or_default();
    let stored_sentinel = existing.lines().find(|l| l.starts_with("SHA512 (md5sums.txt) ="));

    if let Some(stored) = stored_sentinel {
        if stored.trim() == current_sentinel.trim() {
            info!("sentinel unchanged — sources are up to date");
            return CheckResult::UpToDate;
        }
        warn!(stored = %stored, current = %current_sentinel, "sentinel mismatch");
    } else {
        info!("no sentinel found — generating fresh");
    }

    if dry_run {
        info!("dry run — skipping archive downloads");
        return CheckResult::Changed {
            new_sources: current_sentinel,
        };
    }

    // Re-download all archives and compute SHA-512
    info!("recomputing SHA-512 for all modules");

    let mut lines: Vec<String> = vec![current_sentinel];
    let mut failures: Vec<String> = Vec::new();

    for module in MODULES {
        let filename = format!("{module}-everywhere-src-{version}.tar.xz");
        let url = format!("{BASE_URL}/{major_minor}/{version}/submodules/{filename}");
        debug!(module, %url, "downloading");

        match download_and_hash(&client, &url) {
            Ok(hash) => {
                info!(module, "ok");
                lines.push(format!("SHA512 ({filename}) = {hash}"));
            }
            Err(e) => {
                error!(module, %e, "download failed");
                failures.push(format!("{filename}: {e}"));
            }
        }
    }

    if !failures.is_empty() {
        warn!(count = failures.len(), "some modules failed");
        for f in &failures {
            error!(failure = %f);
        }
    }

    CheckResult::Changed {
        new_sources: lines.join("\n"),
    }
}

#[allow(clippy::missing_errors_doc)]
pub fn download_and_hash(client: &reqwest::blocking::Client, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut resp = client.get(url).send()?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()).into());
    }

    let mut hasher = Sha512::new();
    let mut buf = vec![0u8; 64 * 1024];

    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}
