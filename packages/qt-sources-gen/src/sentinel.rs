use std::collections::HashMap;

use sha2::{Digest, Sha512};
use tracing::info;

use crate::error::Error;
use crate::sources::SourcesFile;
use crate::{client, modules};

/// The result of fetching and hashing `md5sums.txt`.
pub struct Sentinel {
    /// SHA-512 hash of the raw md5sums.txt content.
    pub hash: String,
    /// Parsed per-file MD5 expectations from md5sums.txt.
    pub expected_md5s: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SentinelStatus {
    Unchanged,
    Changed,
    Missing,
}

/// # Errors
/// Returns an error if the version format is invalid or `md5sums.txt` cannot be fetched.
pub fn fetch_sentinel(client: &reqwest::blocking::Client, version: &str) -> Result<Sentinel, Error> {
    let (major_minor, _) = modules::split_version(version).ok_or_else(|| Error::InvalidVersion(version.to_string()))?;

    let url = modules::md5sums_url(major_minor, version);
    info!(url = %url, "fetching md5sums.txt");

    let body = client::download_body(client, &url)?;

    let hash = hex::encode(Sha512::digest(&body));
    let expected_md5s = parse_md5sums(&body);
    info!(modules = expected_md5s.len(), "parsed md5sums.txt");

    Ok(Sentinel { hash, expected_md5s })
}

#[must_use]
pub fn compare_sentinel(sentinel: &Sentinel, sources: &SourcesFile) -> SentinelStatus {
    match &sources.sentinel {
        Some(stored) if *stored == sentinel.hash => SentinelStatus::Unchanged,
        Some(_) => SentinelStatus::Changed,
        None => SentinelStatus::Missing,
    }
}

/// Parse md5sums.txt content into a filename → MD5 map.
fn parse_md5sums(body: &[u8]) -> HashMap<String, String> {
    let text = std::str::from_utf8(body).unwrap_or("");
    let mut map = HashMap::new();
    for line in text.lines() {
        // Format: "<md5hash>  <filename>" (two spaces between hash and name)
        if let Some((hash, name)) = line.split_once("  ") {
            let hash = hash.trim();
            let name = name.trim();
            if hash.len() == 32 && !name.is_empty() {
                map.insert(name.to_string(), hash.to_string());
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_md5sums_valid() {
        let body = b"d41d8cd98f00b204e9800998ecf8427e  qtbase-everywhere-src-6.11.0.tar.xz\n\
                      0cc175b9c0f1b6a831c399e269772661  qtsvg-everywhere-src-6.11.0.tar.xz\n";
        let map = parse_md5sums(body);
        assert_eq!(map.len(), 2);
        assert_eq!(map["qtbase-everywhere-src-6.11.0.tar.xz"], "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn parse_md5sums_ignores_bad_lines() {
        let body = b"short  file.tar.xz\n\
                      not-a-valid-line\n\
                      d41d8cd98f00b204e9800998ecf8427e  good.tar.xz\n";
        let map = parse_md5sums(body);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("good.tar.xz"));
    }

    #[test]
    fn compare_sentinel_unchanged() {
        let sentinel = Sentinel {
            hash: "abc123".into(),
            expected_md5s: HashMap::new(),
        };
        let sources = SourcesFile {
            sentinel: Some("abc123".into()),
            sha512s: HashMap::new(),
        };
        assert_eq!(compare_sentinel(&sentinel, &sources), SentinelStatus::Unchanged);
    }

    #[test]
    fn compare_sentinel_changed() {
        let sentinel = Sentinel {
            hash: "abc123".into(),
            expected_md5s: HashMap::new(),
        };
        let sources = SourcesFile {
            sentinel: Some("old_hash".into()),
            sha512s: HashMap::new(),
        };
        assert_eq!(compare_sentinel(&sentinel, &sources), SentinelStatus::Changed);
    }

    #[test]
    fn compare_sentinel_missing() {
        let sentinel = Sentinel {
            hash: "abc123".into(),
            expected_md5s: HashMap::new(),
        };
        let sources = SourcesFile::default();
        assert_eq!(compare_sentinel(&sentinel, &sources), SentinelStatus::Missing);
    }
}
